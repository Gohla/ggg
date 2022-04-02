#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};

use rayon::{ThreadPool, ThreadPoolBuilder};
use ultraviolet::{Isometry3, Vec3};

use crate::chunk::{ChunkSize, ChunkVertices};
use crate::lod::chunk::{AABB, LodChunkManager, LodChunkVertices};
use crate::marching_cubes::MarchingCubes;
use crate::transvoxel::side::TransitionSide;
use crate::transvoxel::Transvoxel;
use crate::volume::Volume;

#[derive(Copy, Clone, Debug)]
pub struct LodOctmapSettings {
  pub total_size: u32,
  pub lod_factor: f32,
  pub thread_pool_threads: usize,
  pub mesh_cache_size: usize,
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
      mesh_cache_size: 1024,
    }
  }
}

// Octree

pub struct LodOctmap<V, C: ChunkSize> {
  total_size: u32,
  lod_factor: f32,

  transform: Isometry3,
  transform_inversed: Isometry3,

  max_lod_level: u32,
  volume: V,
  marching_cubes: MarchingCubes<C>,
  transvoxel: Transvoxel<C>,

  active_aabbs: HashSet<AABB>,
  keep_aabbs: HashSet<AABB>,
  chunks: HashMap<AABB, (LodChunkVertices, bool)>,

  requested_aabbs: HashSet<AABB>,
  thread_pool: ThreadPool,
  tx: Sender<(AABB, LodChunkVertices)>,
  rx: Receiver<(AABB, LodChunkVertices)>,

  //mesh_cache: LruCache<AABB, Vec<Vertex>>,
}

impl<V: Volume + Clone + Send + 'static, C: ChunkSize> LodOctmap<V, C> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:,
  [u16; MarchingCubes::<C>::SHARED_INDICES_SIZE]:,
  [u16; Transvoxel::<C>::SHARED_INDICES_SIZE]:,
{
  pub fn new(settings: LodOctmapSettings, transform: Isometry3, volume: V, marching_cubes: MarchingCubes<C>, transvoxel: Transvoxel<C>) -> Self {
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
      marching_cubes,
      transvoxel,

      active_aabbs: HashSet::new(),
      keep_aabbs: HashSet::new(),
      chunks: HashMap::new(),

      requested_aabbs: HashSet::new(),
      thread_pool: ThreadPoolBuilder::new().num_threads(settings.thread_pool_threads).build().unwrap_or_else(|e| panic!("Failed to create thread pool: {:?}", e)),
      tx,
      rx,

      //mesh_cache: LruCache::new(settings.mesh_cache_size),
    }
  }

  #[inline]
  pub fn get_max_lod_level(&self) -> u32 { self.max_lod_level }

  pub fn do_update(&mut self, position: Vec3) -> (Isometry3, impl Iterator<Item=(&AABB, &(LodChunkVertices, bool))>) {
    let position = self.transform_inversed.transform_vec(position);

    for (aabb, lod_chunk) in self.rx.try_iter() {
      self.chunks.insert(aabb, (lod_chunk, true));
      self.requested_aabbs.remove(&aabb);
    }

    self.active_aabbs.clear();
    let prev_keep: HashSet<_> = self.keep_aabbs.drain().collect();

    self.update_root_node(position);

    for removed in prev_keep.difference(&self.keep_aabbs) {
      if let Some((mesh, filled)) = self.chunks.get_mut(removed) {
        mesh.clear();
        *filled = false;
      }
    }

    let chunks = self.chunks.iter().filter(|(aabb, _)| self.active_aabbs.contains(*aabb));
    (self.transform, chunks)
  }

  pub fn clear(&mut self) {
    self.keep_aabbs.clear();
    self.active_aabbs.clear();
    for (_, (vertices, filled)) in &mut self.chunks {
      vertices.clear();
      *filled = false;
    }
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
    let (chunk, filled) = self.chunks.entry(aabb).or_default();
    if *filled { return true; }
    if self.requested_aabbs.contains(&aabb) { return false; }
    let chunk = std::mem::take(chunk);
    self.request_chunk(aabb, chunk);
    return false;
  }

  fn request_chunk(&mut self, aabb: AABB, mut chunk: LodChunkVertices) {
    self.requested_aabbs.insert(aabb);
    let total_size = self.total_size;
    let marching_cubes = self.marching_cubes.clone();
    let transvoxel = self.transvoxel.clone();
    let volume = self.volume.clone();
    let tx = self.tx.clone();
    self.thread_pool.spawn(move || {
      let lores_min = aabb.min();
      let lores_max = aabb.max();
      let lores_step = aabb.step::<C>();
      let chunk_samples = volume.sample_chunk(lores_min, lores_step);
      marching_cubes.extract_chunk(lores_min, lores_step, &chunk_samples, &mut chunk.regular);
      if lores_step != 1 { // At max LOD level, no need to create transition cells.
        let hires_step = lores_step / 2;
        if lores_min.x > 0 {
          Self::extract_transvoxel_chunk(aabb, TransitionSide::LoX, &volume, hires_step, lores_step, &transvoxel, &mut chunk.transition_lo_x_chunk);
        }
        if lores_max.x < total_size {
          Self::extract_transvoxel_chunk(aabb, TransitionSide::HiX, &volume, hires_step, lores_step, &transvoxel, &mut chunk.transition_hi_x_chunk);
        }
        if lores_min.y > 0 {
          Self::extract_transvoxel_chunk(aabb, TransitionSide::LoY, &volume, hires_step, lores_step, &transvoxel, &mut chunk.transition_lo_y_chunk);
        }
        if lores_max.y < total_size {
          Self::extract_transvoxel_chunk(aabb, TransitionSide::HiY, &volume, hires_step, lores_step, &transvoxel, &mut chunk.transition_hi_y_chunk);
        }
        if lores_min.z > 0 {
          Self::extract_transvoxel_chunk(aabb, TransitionSide::LoZ, &volume, hires_step, lores_step, &transvoxel, &mut chunk.transition_lo_z_chunk);
        }
        if lores_max.z < total_size {
          Self::extract_transvoxel_chunk(aabb, TransitionSide::HiZ, &volume, hires_step, lores_step, &transvoxel, &mut chunk.transition_hi_z_chunk);
        }
      }
      tx.send((aabb, chunk)).ok(); // Ignore hangups.
    })
  }

  fn extract_transvoxel_chunk(
    aabb: AABB,
    side: TransitionSide,
    volume: &V,
    hires_step: u32,
    lores_step: u32,
    transvoxel: &Transvoxel<C>,
    chunk_vertices: &mut ChunkVertices,
  ) {
    let hires_chunk_mins = side.subdivided_face_of_side_minimums(aabb);
    let hires_chunk_samples = [
      volume.sample_chunk(hires_chunk_mins[0], hires_step),
      volume.sample_chunk(hires_chunk_mins[1], hires_step),
      volume.sample_chunk(hires_chunk_mins[2], hires_step),
      volume.sample_chunk(hires_chunk_mins[3], hires_step),
    ];
    transvoxel.extract_chunk(
      side,
      &hires_chunk_mins,
      &hires_chunk_samples,
      hires_step,
      aabb.min(),
      lores_step,
      chunk_vertices,
    );
  }
}

// Volume-mesh manager abstraction, to enable using Octree without generic arguments.

impl<V: Volume + Clone + Send + 'static, C: ChunkSize> LodChunkManager for LodOctmap<V, C> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:,
  [u16; MarchingCubes::<C>::SHARED_INDICES_SIZE]:,
  [u16; Transvoxel::<C>::SHARED_INDICES_SIZE]:,
{
  #[inline]
  fn get_max_lod_level(&self) -> u32 { self.max_lod_level }
  #[inline]
  fn get_lod_factor(&self) -> f32 { self.lod_factor }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { &mut self.lod_factor }

  #[inline]
  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &(LodChunkVertices, bool))> + '_>) {
    let (transform, chunks) = self.do_update(position);
    (transform, Box::new(chunks))
  }
}
