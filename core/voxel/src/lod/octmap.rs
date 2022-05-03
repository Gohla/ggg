use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};

use lru::LruCache;
use rayon::{ThreadPool, ThreadPoolBuilder};
use ultraviolet::{Isometry3, Vec3};

use crate::chunk::ChunkSize;
use crate::lod::aabb::AABB;
use crate::lod::chunk_mesh::{LodChunkMesh, LodChunkMeshManager, LodChunkMeshManagerParameters};
use crate::lod::extract::LodExtractor;
use crate::volume::Volume;

// Settings

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LodOctmapSettings {
  pub total_size: u32,
  pub lod_factor: f32,
  pub thread_pool_threads: usize,
  pub chunk_mesh_cache_size: usize,
}

impl LodOctmapSettings {
  #[inline]
  pub fn check(&self) {
    assert_ne!(self.total_size, 0, "Total size may not be 0");
    assert!(self.total_size.is_power_of_two(), "Total size {} must be a power of 2", self.total_size);
  }
}

impl Default for LodOctmapSettings {
  fn default() -> Self {
    Self {
      total_size: 4096,
      lod_factor: 1.0,
      thread_pool_threads: 10,
      chunk_mesh_cache_size: 8192,
    }
  }
}

// LOD octmap

pub struct LodOctmap<V: Volume, C: ChunkSize, E: LodExtractor<C>> {
  total_size: u32,
  lod_factor: f32,

  transform: Isometry3,
  transform_inversed: Isometry3,

  max_lod_level: u32,
  volume: V,
  extractor: E,

  active_aabbs: HashSet<AABB>,
  keep_aabbs: HashSet<AABB>,
  lod_chunk_meshes: HashMap<AABB, (E::Chunk, bool)>,

  requested_aabbs: HashSet<AABB>,
  thread_pool: ThreadPool,
  tx: Sender<(AABB, E::Chunk)>,
  rx: Receiver<(AABB, E::Chunk)>,

  chunk_mesh_cache: LruCache<AABB, E::Chunk>,
}

impl<V: Volume, C: ChunkSize, E: LodExtractor<C>> LodOctmap<V, C, E> {
  pub fn new(settings: LodOctmapSettings, transform: Isometry3, volume: V, extractor: E) -> Self {
    settings.check();
    let lod_0_step = settings.total_size / C::CELLS_IN_CHUNK_ROW;
    let max_lod_level = lod_0_step.log2();
    let (tx, rx) = std::sync::mpsc::channel();
    Self {
      total_size: settings.total_size,
      lod_factor: settings.lod_factor,

      transform,
      transform_inversed: transform.inversed(),

      max_lod_level,
      volume,
      extractor,

      active_aabbs: HashSet::new(),
      keep_aabbs: HashSet::new(),
      lod_chunk_meshes: HashMap::new(),

      requested_aabbs: HashSet::new(),
      thread_pool: ThreadPoolBuilder::new().num_threads(settings.thread_pool_threads).build().unwrap_or_else(|e| panic!("Failed to create thread pool: {:?}", e)),
      tx,
      rx,

      chunk_mesh_cache: LruCache::new(settings.chunk_mesh_cache_size),
    }
  }

  #[inline]
  pub fn get_max_lod_level(&self) -> u32 { self.max_lod_level }

  pub fn update(&mut self, position: Vec3) -> (Isometry3, impl Iterator<Item=(&AABB, &(E::Chunk, bool))>) {
    let position = self.transform_inversed.transform_vec(position);

    for (aabb, lod_chunk_mesh) in self.rx.try_iter() {
      self.lod_chunk_meshes.insert(aabb, (lod_chunk_mesh, true));
      self.requested_aabbs.remove(&aabb);
    }

    self.active_aabbs.clear();
    let prev_keep: HashSet<_> = self.keep_aabbs.drain().collect();

    self.update_root_node(position);

    for removed in prev_keep.difference(&self.keep_aabbs) {
      if let Some((lod_chunk_mesh, filled)) = self.lod_chunk_meshes.remove(removed) {
        if filled {
          self.chunk_mesh_cache.put(*removed, lod_chunk_mesh);
          // TODO: reuse the chunk mush returned by push by clearing it and using it for new chunk meshes?
        }
      }
    }

    let chunks = self.lod_chunk_meshes.iter().filter(|(aabb, _)| self.active_aabbs.contains(*aabb));
    (self.transform, chunks)
  }

  pub fn clear(&mut self) {
    self.keep_aabbs.clear();
    self.active_aabbs.clear();
    for (_, (vertices, filled)) in &mut self.lod_chunk_meshes {
      vertices.clear();
      *filled = false;
    }
    self.chunk_mesh_cache.clear();
  }

  fn update_root_node(&mut self, position: Vec3) {
    let root = AABB::from_size(self.total_size);
    let (filled, activated) = self.update_nodes(root, 0, position);
    if filled && !activated {
      self.active_aabbs.insert(root);
    }
  }

  fn update_nodes(&mut self, aabb: AABB, lod_level: u32, position: Vec3) -> (bool, bool) {
    self.keep_aabbs.insert(aabb);
    let self_filled = self.update_chunk(aabb);
    if self.is_terminal(aabb, lod_level, position) {
      (self_filled, false)
    } else { // Subdivide
      let mut all_filled = true;
      let subdivided = aabb.subdivide();
      let mut activated = [false; 8];
      for (i, sub_aabb) in subdivided.into_iter().enumerate() {
        let (sub_filled, sub_activated) = self.update_nodes(sub_aabb, lod_level + 1, position);
        activated[i] = sub_activated;
        all_filled &= sub_filled;
      }
      if all_filled {
        for (i, sub_aabb) in subdivided.into_iter().enumerate() {
          if !activated[i] {
            self.active_aabbs.insert(sub_aabb);
          }
        }
        (true, true)
      } else {
        if self_filled {
          self.active_aabbs.insert(aabb);
        }
        (self_filled, self_filled)
      }
    }
  }

  #[inline]
  fn is_terminal(&self, aabb: AABB, lod_level: u32, position: Vec3) -> bool {
    lod_level >= self.max_lod_level || aabb.distance_from(position) > self.lod_factor * aabb.size() as f32
  }

  fn update_chunk(&mut self, aabb: AABB) -> bool {
    let (chunk_mesh, filled) = self.lod_chunk_meshes.entry(aabb).or_default();
    if *filled { return true; }
    if self.requested_aabbs.contains(&aabb) { return false; }
    if let Some(cached_chunk_mesh) = self.chunk_mesh_cache.pop(&aabb) {
      *chunk_mesh = cached_chunk_mesh;
      *filled = true;
      return true;
    }
    let chunk = std::mem::take(chunk_mesh);
    self.request_chunk(aabb, chunk);
    return false;
  }

  fn request_chunk(&mut self, aabb: AABB, mut chunk: E::Chunk) {
    self.requested_aabbs.insert(aabb);
    let total_size = self.total_size;
    let volume = self.volume.clone();
    let extractor = self.extractor.clone();
    let tx = self.tx.clone();
    self.thread_pool.spawn(move || {
      extractor.extract::<V>(total_size, aabb, &volume, &mut chunk);
      tx.send((aabb, chunk)).ok(); // Ignore hangups.
    })
  }
}

// Trait implementation

impl<V: Volume, C: ChunkSize, E: LodExtractor<C>> LodChunkMeshManager<C> for LodOctmap<V, C, E> {
  type Extractor = E;

  #[inline]
  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &(<<Self as LodChunkMeshManager<C>>::Extractor as LodExtractor<C>>::Chunk, bool))> + '_>) {
    let (transform, chunks) = self.update(position);
    (transform, Box::new(chunks))
  }

  #[inline]
  fn get_extractor(&self) -> &E {
    &self.extractor
  }
}

impl<V: Volume, C: ChunkSize, E: LodExtractor<C>> LodChunkMeshManagerParameters for LodOctmap<V, C, E> {
  #[inline]
  fn get_max_lod_level(&self) -> u32 { self.max_lod_level }
  #[inline]
  fn get_lod_factor(&self) -> f32 { self.lod_factor }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { &mut self.lod_factor }
}
