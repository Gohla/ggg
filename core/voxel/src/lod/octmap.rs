use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};
use ultraviolet::{Isometry3, Vec3};

use job_queue::{In, JobQueue, JobQueueMessage};

use crate::chunk::sample::ChunkSamples;
use crate::chunk::size::ChunkSize;
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
  pub fixed_lod_level: Option<u8>,
  pub job_queue_worker_threads: usize,
  pub empty_lod_chunk_mesh_cache_size: usize,
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
      fixed_lod_level: None,
      job_queue_worker_threads: std::thread::available_parallelism().ok().and_then(|p| NonZeroUsize::new(p.get().saturating_sub(1))).unwrap_or(NonZeroUsize::new(7).unwrap()).get(),
      empty_lod_chunk_mesh_cache_size: 4096,
    }
  }
}

// LOD octmap

pub struct LodOctmap<V: Volume, C: ChunkSize, E: LodExtractor<C>> {
  total_size: u32,
  lod_factor: f32,
  fixed_lod_level: Option<u8>,

  transform: Isometry3,
  transform_inversed: Isometry3,

  max_lod_level: u8,
  volume: V,
  extractor: E,

  active_aabbs: FxHashSet<AABB>,
  keep_aabbs: FxHashSet<AABB>,
  lod_chunk_meshes: FxHashMap<AABB, Arc<E::Chunk>>,
  empty_lod_chunk_mesh_cache: VecDeque<E::Chunk>,
  empty_lod_chunk_mesh_cache_size: usize,

  requested_aabbs: FxHashSet<AABB>,
  job_queue: JobQueue<LodJobKey, E::JobDepKey, LodJobInput<V, E::Chunk>, LodJobOutput<ChunkSamples<C>, E::Chunk>, 7>,
}

impl<V: Volume, C: ChunkSize, E: LodExtractor<C>> LodOctmap<V, C, E> {
  pub fn new(settings: LodOctmapSettings, transform: Isometry3, volume: V, extractor: E) -> Self {
    settings.check();
    let lod_0_step = settings.total_size / C::CELLS_IN_CHUNK_ROW;
    let max_lod_level = lod_0_step.log2() as u8;
    Self {
      total_size: settings.total_size,
      lod_factor: settings.lod_factor,
      fixed_lod_level: settings.fixed_lod_level,

      transform,
      transform_inversed: transform.inversed(),

      max_lod_level,
      volume,
      extractor: extractor.clone(),

      active_aabbs: FxHashSet::default(),
      keep_aabbs: FxHashSet::default(),
      lod_chunk_meshes: FxHashMap::default(),
      empty_lod_chunk_mesh_cache: VecDeque::with_capacity(settings.empty_lod_chunk_mesh_cache_size),
      empty_lod_chunk_mesh_cache_size: settings.empty_lod_chunk_mesh_cache_size,

      requested_aabbs: FxHashSet::default(),
      job_queue: JobQueue::new(settings.job_queue_worker_threads, move |job_key: LodJobKey, dependency_outputs, input: LodJobInput<V, E::Chunk>| {
        match (job_key, input) {
          (LodJobKey::Sample(aabb), LodJobInput::Sample(volume)) => {
            LodJobOutput::Sample(Arc::new(volume.sample_chunk(aabb.min, aabb.step::<C>())))
          }
          (LodJobKey::Mesh(aabb), LodJobInput::Mesh { total_size, mut lod_chunk_mesh }) => {
            extractor.run_job(total_size, aabb, dependency_outputs, &mut lod_chunk_mesh);
            LodJobOutput::Mesh(Arc::new(lod_chunk_mesh))
          }
          _ => { panic!("Received non-matching job key and input") }
        }
      }).unwrap_or_else(|e| panic!("Failed to create job queue: {:?}", e)),
    }
  }

  #[inline]
  pub fn get_max_lod_level(&self) -> u8 { self.max_lod_level }

  #[profiling::function]
  pub fn update(&mut self, position: Vec3) -> (Isometry3, impl Iterator<Item=(&AABB, &Arc<E::Chunk>)>) {
    let position = self.transform_inversed.transform_vec(position);

    for message in self.job_queue.get_message_receiver().try_iter() {
      match message {
        JobQueueMessage::JobCompleted(job_key, output) => {
          if let (LodJobKey::Mesh(aabb), LodJobOutput::Mesh(arc)) = (job_key, output) {
            self.lod_chunk_meshes.insert(aabb, arc);
            self.requested_aabbs.remove(&aabb);
          }
        }
        JobQueueMessage::CompletedJobRemoved(_, output) => {
          if let LodJobOutput::Mesh(arc) = output {
            Self::cache_empty_lod_chunk_mesh(&mut self.empty_lod_chunk_mesh_cache, self.empty_lod_chunk_mesh_cache_size, arc);
          }
        }
        _ => {}
      }
    }

    self.active_aabbs.clear();
    let prev_keep: FxHashSet<_> = self.keep_aabbs.drain().collect(); // OPTO: clear and collect into existing hashset.

    self.update_root_node(position);

    let mut send_error = false;
    for removed in prev_keep.difference(&self.keep_aabbs) {
      self.requested_aabbs.remove(&removed);
      send_error |= self.job_queue.try_remove_job_and_orphaned_dependencies(LodJobKey::Mesh(*removed)).is_err();
      if send_error { break; }
      if let Some(arc) = self.lod_chunk_meshes.remove(removed) {
        Self::cache_empty_lod_chunk_mesh(&mut self.empty_lod_chunk_mesh_cache, self.empty_lod_chunk_mesh_cache_size, arc);
      }
    }
    if send_error {
      self.handle_send_error();
    }

    let chunks = self.lod_chunk_meshes.iter().filter(|(aabb, _)| self.active_aabbs.contains(*aabb));
    (self.transform, chunks)
  }

  pub fn clear(&mut self) {
    self.keep_aabbs.clear();
    self.active_aabbs.clear();
    self.lod_chunk_meshes.clear();
  }

  #[inline]
  fn cache_empty_lod_chunk_mesh(empty_lod_chunk_mesh_cache: &mut VecDeque<E::Chunk>, empty_lod_chunk_mesh_cache_size: usize, arc: Arc<E::Chunk>) {
    if empty_lod_chunk_mesh_cache.len() >= empty_lod_chunk_mesh_cache_size { return; }
    if let Ok(mut lod_chunk_mesh) = Arc::try_unwrap(arc) {
      lod_chunk_mesh.clear();
      empty_lod_chunk_mesh_cache.push_back(lod_chunk_mesh);
    }
  }


  #[profiling::function]
  fn update_root_node(&mut self, position: Vec3) {
    let root = AABB::from_size(self.total_size);
    let lod_level = 0;
    let neighbor_lods = NeighborLods::from_single_lod_level(lod_level);
    let NodeResult { filled, activated, .. } = self.update_nodes(root, lod_level, neighbor_lods, position);
    if filled && !activated {
      self.active_aabbs.insert(root);
    }
  }

  #[profiling::function]
  fn update_nodes(&mut self, aabb: AABB, lod_level: u8, neighbor_lods: NeighborLods, position: Vec3) -> NodeResult {
    self.keep_aabbs.insert(aabb);
    let self_filled = self.update_chunk(aabb);
    if self.is_terminal(aabb, lod_level, neighbor_lods.minimum_lod_level(), position) {
      NodeResult::new(self_filled, false, NeighborLods::from_single_lod_level(lod_level))
    } else { // Subdivide
      let subdivided @ [front, front_x, front_y, front_xy, back, back_x, back_y, back_xy] = aabb.subdivide();
      let mut all_filled = true;
      let mut activated = [false; 8];
      let lod_level_plus_one = lod_level + 1;

      // Front
      let front_result = {
        let result = self.update_nodes(front, lod_level_plus_one, neighbor_lods, position);
        activated[0] = result.activated;
        all_filled &= result.filled;
        result
      };
      // Front X (left of Front)
      let front_x_result = {
        let mut neighbor_lods = neighbor_lods;
        neighbor_lods.x = front_result.neighbor_lods.x;
        let result = self.update_nodes(front_x, lod_level_plus_one, neighbor_lods, position);
        activated[1] = result.activated;
        all_filled &= result.filled;
        result
      };
      // Front Y (down of Front)
      let front_y_result = {
        let mut neighbor_lods = neighbor_lods;
        neighbor_lods.y = front_result.neighbor_lods.y;
        let result = self.update_nodes(front_y, lod_level_plus_one, neighbor_lods, position);
        activated[2] = result.activated;
        all_filled &= result.filled;
        result
      };
      // Front XY (left and down of Front)
      let front_xy_result = {
        let mut neighbor_lods = neighbor_lods;
        neighbor_lods.x = front_y_result.neighbor_lods.x;
        neighbor_lods.y = front_x_result.neighbor_lods.y;
        neighbor_lods.xy = front_result.neighbor_lods.xy;
        let result = self.update_nodes(front_xy, lod_level_plus_one, neighbor_lods, position);
        activated[3] = result.activated;
        all_filled &= result.filled;
        result
      };
      // Back
      let back_result = {
        let mut neighbor_lods = neighbor_lods;
        neighbor_lods.z = front_result.neighbor_lods.z;
        let result = self.update_nodes(back, lod_level_plus_one, neighbor_lods, position);
        activated[4] = result.activated;
        all_filled &= result.filled;
        result
      };
      // Back X (left of Back)
      let back_x_result = {
        let mut neighbor_lods = neighbor_lods;
        neighbor_lods.x = back_result.neighbor_lods.x;
        neighbor_lods.z = front_x_result.neighbor_lods.z;
        neighbor_lods.xz = front_result.neighbor_lods.xz;
        let result = self.update_nodes(back_x, lod_level_plus_one, neighbor_lods, position);
        activated[5] = result.activated;
        all_filled &= result.filled;
        result
      };
      // Back Y (down of Back)
      let back_y_result = {
        let mut neighbor_lods = neighbor_lods;
        neighbor_lods.y = back_result.neighbor_lods.y;
        neighbor_lods.z = front_y_result.neighbor_lods.z;
        neighbor_lods.yz = front_result.neighbor_lods.yz;
        let result = self.update_nodes(back_y, lod_level_plus_one, neighbor_lods, position);
        activated[6] = result.activated;
        all_filled &= result.filled;
        result
      };
      // Back XY (left and down of Back)
      let back_xy_result = {
        let mut neighbor_lods = neighbor_lods;
        neighbor_lods.x = back_y_result.neighbor_lods.x;
        neighbor_lods.y = back_x_result.neighbor_lods.y;
        neighbor_lods.xy = back_result.neighbor_lods.xy;
        neighbor_lods.yz = front_x_result.neighbor_lods.yz;
        neighbor_lods.xz = front_y_result.neighbor_lods.xz;
        let result = self.update_nodes(back_xy, lod_level_plus_one, neighbor_lods, position);
        activated[7] = result.activated;
        all_filled &= result.filled;
        result
      };

      let neighbor_lods = {
        let x = front_x_result.neighbor_lods.x.max(front_xy_result.neighbor_lods.x).max(back_x_result.neighbor_lods.x).max(back_xy_result.neighbor_lods.x);
        let y = front_y_result.neighbor_lods.y.max(front_xy_result.neighbor_lods.y).max(back_y_result.neighbor_lods.y).max(back_xy_result.neighbor_lods.y);
        let z = back_result.neighbor_lods.z.max(back_x_result.neighbor_lods.z).max(back_y_result.neighbor_lods.z).max(back_xy_result.neighbor_lods.z);
        let xy = back_xy_result.neighbor_lods.xy.max(front_xy_result.neighbor_lods.xy);
        let yz = front_y_result.neighbor_lods.yz.max(front_xy_result.neighbor_lods.yz);
        let xz = front_x_result.neighbor_lods.xz.max(front_xy_result.neighbor_lods.xz);
        NeighborLods::new(x, y, z, xy, yz, xz)
      };

      if all_filled { // All subdivided nodes are filled, activate each non-activated node.
        for (i, sub_aabb) in subdivided.into_iter().enumerate() {
          if !activated[i] {
            self.active_aabbs.insert(sub_aabb);
          }
        }
        NodeResult::new(true, true, neighbor_lods) // Act as is filled and activated, because all sub-nodes are filled and activated.
      } else {
        NodeResult::new(self_filled, false, neighbor_lods) // Not all subdivided nodes are filled, we might be filled. Our parent should activate us if possible.
      }
    }
  }

  #[inline]
  fn is_terminal(&self, aabb: AABB, lod_level: u8, minimum_lod_level: u8, position: Vec3) -> bool {
    if let Some(fixed_lod_level) = self.fixed_lod_level {
      lod_level >= self.max_lod_level.min(fixed_lod_level)
    } else {
      (lod_level >= minimum_lod_level) && (lod_level >= self.max_lod_level || aabb.distance_from(position) > self.lod_factor * aabb.size as f32)
    }
  }

  #[profiling::function]
  fn update_chunk(&mut self, aabb: AABB) -> bool {
    if self.lod_chunk_meshes.contains_key(&aabb) { return true; }
    if !self.requested_aabbs.contains(&aabb) {
      let empty_lod_chunk_mesh = self.empty_lod_chunk_mesh_cache.pop_front().unwrap_or_else(|| E::Chunk::default());
      self.extractor.create_jobs(self.total_size, aabb, self.volume.clone(), empty_lod_chunk_mesh, &self.job_queue)
        .unwrap_or_else(|_| self.handle_send_error());
      self.requested_aabbs.insert(aabb);
    }
    return false;
  }


  fn handle_send_error(&mut self) {
    if let Err(e) = self.job_queue.take_and_join() {
      std::panic::resume_unwind(e);
    } else {
      panic!("Communicating with the job queue failed, but it did not panic");
    }
  }
}


// Octmap algorithm return type

struct NodeResult {
  filled: bool,
  activated: bool,
  neighbor_lods: NeighborLods,
}

#[derive(Copy, Clone)]
struct NeighborLods {
  // LOD level of the X-neighbor; the left side
  x: u8,
  // LOD level of the Y-neighbor; the bottom side
  y: u8,
  // LOD level of the Z-neighbor; the back side
  z: u8,
  xy: u8,
  yz: u8,
  xz: u8,
}

impl NeighborLods {
  #[inline]
  fn new(x: u8, y: u8, z: u8, xy: u8, yz: u8, xz: u8) -> Self { Self { x, y, z, xy, yz, xz } }
  #[inline]
  fn from_single_lod_level(l: u8) -> Self { Self { x: l, y: l, z: l, xy: l, yz: l, xz: l } }

  #[inline]
  fn max(&self) -> u8 { self.x.max(self.y).max(self.z).max(self.xy).max(self.yz).max(self.xz) }
  #[inline]
  fn minimum_lod_level(&self) -> u8 { self.max().saturating_sub(1) }
}

impl NodeResult {
  #[inline]
  fn new(filled: bool, activated: bool, neighbor_lods: NeighborLods) -> Self {
    Self { filled, activated, neighbor_lods }
  }
}


// Job types

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum LodJobKey {
  Sample(AABB),
  Mesh(AABB),
}

pub enum LodJobInput<V: In, CM: In> {
  Sample(V),
  Mesh { total_size: u32, lod_chunk_mesh: CM },
}

pub enum LodJobOutput<CS, CM> {
  Sample(Arc<CS>),
  Mesh(Arc<CM>),
}

impl<CS, CM> Clone for LodJobOutput<CS, CM> {
  #[inline]
  fn clone(&self) -> Self {
    match self {
      LodJobOutput::Sample(arc) => LodJobOutput::Sample(arc.clone()),
      LodJobOutput::Mesh(arc) => LodJobOutput::Mesh(arc.clone()),
    }
  }
}


// LodChunkMeshManager trait implementation

impl<V: Volume, C: ChunkSize, E: LodExtractor<C>> LodChunkMeshManager<C> for LodOctmap<V, C, E> {
  type Extractor = E;
  #[inline]
  fn get_extractor(&self) -> &E {
    &self.extractor
  }

  #[inline]
  fn update(&mut self, position: Vec3) -> (Isometry3, Box<dyn Iterator<Item=(&AABB, &Arc<E::Chunk>)> + '_>) {
    let (transform, chunks) = self.update(position);
    (transform, Box::new(chunks))
  }
}

impl<V: Volume, C: ChunkSize, E: LodExtractor<C>> LodChunkMeshManagerParameters for LodOctmap<V, C, E> {
  #[inline]
  fn get_max_lod_level(&self) -> u8 { self.max_lod_level }

  #[inline]
  fn get_lod_factor(&self) -> f32 { self.lod_factor }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { &mut self.lod_factor }

  #[inline]
  fn get_fixed_lod_level(&self) -> Option<u8> { self.fixed_lod_level }
  #[inline]
  fn get_fixed_lod_level_mut(&mut self) -> &mut Option<u8> { &mut self.fixed_lod_level }
}
