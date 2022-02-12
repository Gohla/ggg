#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};

use rayon::{ThreadPool, ThreadPoolBuilder};
use ultraviolet::{UVec3, Vec3};

use crate::chunk::{CELLS_IN_CHUNK_ROW, LodChunk};
use crate::marching_cubes::MarchingCubes;
use crate::transvoxel::{TransitionSide, Transvoxel};
use crate::volume::Volume;

// Trait

pub trait VolumeMeshManager {
  fn get_max_lod_level(&self) -> u32;
  fn get_lod_factor(&self) -> f32;
  fn get_lod_factor_mut(&mut self) -> &mut f32;

  fn update(&mut self, position: Vec3) -> Box<dyn Iterator<Item=(&AABB, &(LodChunk, bool))> + '_>;
}

// Octree settings

#[derive(Copy, Clone, Debug)]
pub struct OctreeSettings {
  pub total_size: u32,
  pub lod_factor: f32,
  pub thread_pool_threads: usize,
  pub mesh_cache_size: usize,
}

impl OctreeSettings {
  #[inline]
  pub fn check(&self) {
    assert_ne!(self.total_size, 0, "Total size may not be 0");
    assert!(self.total_size.is_power_of_two(), "Total size {} must be a power of 2", self.total_size);
  }
}

impl Default for OctreeSettings {
  fn default() -> Self {
    Self {
      total_size: 4096,
      lod_factor: 1.0,
      thread_pool_threads: 5,
      mesh_cache_size: 1024,
    }
  }
}

// Octree

pub struct Octree<V> {
  total_size: u32,
  lod_factor: f32,

  max_lod_level: u32,
  volume: V,
  marching_cubes: MarchingCubes,
  transvoxel: Transvoxel,

  active_aabbs: HashSet<AABB>,
  keep_aabbs: HashSet<AABB>,
  chunks: HashMap<AABB, (LodChunk, bool)>,

  requested_aabbs: HashSet<AABB>,
  thread_pool: ThreadPool,
  tx: Sender<(AABB, LodChunk)>,
  rx: Receiver<(AABB, LodChunk)>,

  //mesh_cache: LruCache<AABB, Vec<Vertex>>,
}

impl<V: Volume + Clone + Send + 'static> Octree<V> {
  pub fn new(settings: OctreeSettings, volume: V, marching_cubes: MarchingCubes, transvoxel: Transvoxel) -> Self {
    settings.check();
    let lod_0_step = settings.total_size / CELLS_IN_CHUNK_ROW;
    let max_lod_level = lod_0_step.log2();
    let (tx, rx) = std::sync::mpsc::channel();
    Self {
      total_size: settings.total_size,
      lod_factor: settings.lod_factor,
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

  pub fn update(&mut self, position: Vec3) -> impl Iterator<Item=(&AABB, &(LodChunk, bool))> {
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

    self.chunks.iter().filter(|(aabb, _)| self.active_aabbs.contains(*aabb))
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

  fn request_chunk(&mut self, aabb: AABB, mut chunk: LodChunk) {
    self.requested_aabbs.insert(aabb);
    let marching_cubes = self.marching_cubes.clone();
    let transvoxel = self.transvoxel.clone();
    let volume = self.volume.clone();
    let tx = self.tx.clone();
    self.thread_pool.spawn(move || {
      let lores_min = aabb.min;
      let lores_step = aabb.size() / CELLS_IN_CHUNK_ROW;
      let chunk_samples = volume.sample_chunk(lores_min, lores_step);
      marching_cubes.extract_chunk(lores_min, lores_step, &chunk_samples, &mut chunk.regular);
      // HACK: transvoxel
      if lores_step != 1 { // At max LOD level, no need to create transition cells.
        let hires_step = lores_step / 2;
        if lores_min.z > 0 {
          let side = TransitionSide::LowZ;
          let hires_chunk_mins = aabb.subdivided_face_of_side_minimums(side);
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
            lores_min,
            lores_step,
            &mut chunk.transition_low_z_chunk,
          );
        }
      }
      tx.send((aabb, chunk)).ok(); // Ignore hangups.
    })
  }
}

// Volume-mesh manager abstraction, to enable using Octree without generic arguments.

impl<V: Volume + Clone + Send + 'static> VolumeMeshManager for Octree<V> {
  #[inline]
  fn get_max_lod_level(&self) -> u32 { self.max_lod_level }
  #[inline]
  fn get_lod_factor(&self) -> f32 { self.lod_factor }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { &mut self.lod_factor }

  #[inline]
  fn update(&mut self, position: Vec3) -> Box<dyn Iterator<Item=(&AABB, &(LodChunk, bool))> + '_> { Box::new(self.update(position)) }
}

// AABB

/// Square axis-aligned bounding box, always in powers of 2, and with size always larger than 1.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct AABB {
  min: UVec3,
  size: u32,
}

impl AABB {
  #[inline]
  pub fn from_size(size: u32) -> Self {
    assert_ne!(size, 0, "Size may not be 0");
    assert_ne!(size, 1, "Size may not be 1");
    assert!(size.is_power_of_two(), "Size {} must be a power of 2", size);
    let min = UVec3::new(0, 0, 0);
    Self { min, size }
  }

  #[inline(always)]
  pub fn min(&self) -> UVec3 { self.min }

  #[inline(always)]
  pub fn size(&self) -> u32 { self.size }

  #[inline(always)]
  pub fn size_3d(&self) -> UVec3 { UVec3::new(self.size, self.size, self.size) }

  #[inline(always)]
  pub fn max(&self) -> UVec3 { self.min + self.size_3d() }

  #[inline]
  pub fn extends(&self) -> u32 {
    self.size() / 2 // Note: no rounding needed because AABB is always size of 2 and > 1.
  }

  #[inline]
  pub fn extends_3d(&self) -> UVec3 {
    let extends = self.extends();
    UVec3::new(extends, extends, extends)
  }

  #[inline]
  pub fn center(&self) -> UVec3 {
    self.min + self.extends_3d()
  }

  #[inline]
  pub fn distance_from(&self, point: Vec3) -> f32 {
    // TODO: copied from voxel-planets, check if this is correct and efficient?
    let distance_to_center = (point - self.center().into()).abs();
    let extends = self.extends_3d().into();
    let v = Vec3::zero().max_by_component(distance_to_center - extends).map(|f| f.powf(2.0));
    let distance = (v.x + v.y + v.z).sqrt();
    distance
  }

  #[inline]
  pub fn subdivide(&self) -> [AABB; 8] {
    let min = self.min;
    let cen = self.center();
    let extends = self.extends();
    [
      Self::new_unchecked(min, extends),
      Self::new_unchecked(UVec3::new(min.x, cen.y, min.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, min.y, min.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, cen.y, min.z), extends),
      Self::new_unchecked(UVec3::new(min.x, min.y, cen.z), extends),
      Self::new_unchecked(UVec3::new(min.x, cen.y, cen.z), extends),
      Self::new_unchecked(UVec3::new(cen.x, min.y, cen.z), extends),
      Self::new_unchecked(cen, extends),
    ]
  }

  #[inline]
  pub fn subdivided_face_of_side_minimums(&self, side: TransitionSide) -> [UVec3; 4] {
    match side {
      TransitionSide::LowX => {
        todo!()
      }
      TransitionSide::HighX => {
        todo!()
      }
      TransitionSide::LowY => {
        todo!()
      }
      TransitionSide::HighY => {
        todo!()
      }
      TransitionSide::LowZ => {
        let min = self.min;
        let cen = self.center();
        let extends = self.extends();
        let z = min.z - extends;
        [
          UVec3::new(min.x, min.y, z),
          UVec3::new(cen.x, min.y, z),
          UVec3::new(min.x, cen.y, z),
          UVec3::new(cen.x, cen.y, z),
        ]
      }
      TransitionSide::HighZ => {
        todo!()
      }
    }
  }


  #[inline(always)]
  fn new_unchecked(min: UVec3, size: u32) -> Self {
    Self { min, size }
  }
}
