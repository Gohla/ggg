use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::sync::Arc;

use profiling::scope;
use rustc_hash::{FxHashMap, FxHashSet};
use ultraviolet::{Isometry3, Vec3};

use job_queue::{Job, JobQueue, JobQueueMessage};

use crate::chunk::sample::MaybeCompressedChunkSampleArray;
use crate::chunk::size::ChunkSize;
use crate::lod::aabb::{Aabb, AabbSubdivide, PerAabbSubdivide};
use crate::lod::chunk_mesh::{LodChunkMesh, LodChunkMeshManager, LodChunkMeshManagerParameters};
use crate::lod::extract::{LodExtractor, NeighborDepths};
use crate::volume::Volume;

// Settings

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LodOctmapSettings {
  pub root_size: u32,
  pub lod_factor: f32,
  pub fixed_lod_level: Option<u8>,
  pub job_queue_worker_threads: usize,
  pub empty_lod_chunk_mesh_cache_size: usize,
}
impl LodOctmapSettings {
  #[inline]
  pub fn check(&self) {
    assert_ne!(self.root_size, 0, "Root size may not be 0");
    assert!(self.root_size.is_power_of_two(), "Root size {} must be a power of 2", self.root_size);
  }
}
impl Default for LodOctmapSettings {
  fn default() -> Self {
    Self {
      root_size: 4096,
      lod_factor: 1.0,
      fixed_lod_level: None,
      job_queue_worker_threads: std::thread::available_parallelism().ok().and_then(|p| NonZeroUsize::new(p.get().saturating_sub(1))).unwrap_or(NonZeroUsize::new(7).unwrap()).get(),
      empty_lod_chunk_mesh_cache_size: 4096,
    }
  }
}

// LOD octmap

pub struct LodOctmap<C: ChunkSize, V: Volume, E: LodExtractor<C>> {
  root_size: u32,
  lod_factor: f32,
  fixed_lod_level: Option<u8>,

  transform: Isometry3,
  transform_inversed: Isometry3,

  max_depth: u8,
  volume: V,
  extractor: E,

  active_aabbs: FxHashSet<Aabb>,
  keep_aabbs: FxHashSet<Aabb>,
  prev_keep_aabbs: FxHashSet<Aabb>,
  lod_chunk_meshes: FxHashMap<Aabb, Arc<E::Chunk>>,
  empty_lod_chunk_mesh_cache: VecDeque<E::Chunk>,
  empty_lod_chunk_mesh_cache_size: usize,

  requested_meshing: FxHashSet<Aabb>,
  requested_removal: FxHashSet<Aabb>,
  job_queue: JobQueue<Aabb, E::DependencyKey, LodJobInput<V, E::JobInput>, LodJob<C, V, E>, LodJobOutput<MaybeCompressedChunkSampleArray<C>, E::Chunk>>,
}

impl<C: ChunkSize, V: Volume, E: LodExtractor<C>> LodOctmap<C, V, E> {
  pub fn new(settings: LodOctmapSettings, transform: Isometry3, volume: V, extractor: E) -> Self {
    settings.check();
    let root_size = settings.root_size;
    let lod_0_step = root_size / C::CELLS_IN_CHUNK_ROW;
    let max_depth = lod_0_step.ilog2() as u8;
    Self {
      root_size,
      lod_factor: settings.lod_factor,
      fixed_lod_level: settings.fixed_lod_level,

      transform,
      transform_inversed: transform.inversed(),

      max_depth,
      volume,
      extractor: extractor.clone(),

      active_aabbs: FxHashSet::default(),
      keep_aabbs: FxHashSet::default(),
      prev_keep_aabbs: FxHashSet::default(),
      lod_chunk_meshes: FxHashMap::default(),
      empty_lod_chunk_mesh_cache: VecDeque::with_capacity(settings.empty_lod_chunk_mesh_cache_size),
      empty_lod_chunk_mesh_cache_size: settings.empty_lod_chunk_mesh_cache_size,

      requested_meshing: FxHashSet::default(),
      requested_removal: FxHashSet::default(),
      job_queue: JobQueue::new(
        settings.job_queue_worker_threads,
        settings.job_queue_worker_threads * 2,
        4096,
        move |aabb: Aabb, input: LodJobInput<V, E::JobInput>, dependency_outputs: &[(E::DependencyKey, LodJobOutput<MaybeCompressedChunkSampleArray<C>, E::Chunk>)]| {
          match input {
            LodJobInput::Sample(volume) => {
              LodJobOutput::Sample(Arc::new(volume.sample_chunk(aabb.minimum_point(root_size), aabb.step::<C>(root_size))))
            }
            LodJobInput::Mesh(input) => {
              let lod_chunk_mesh = extractor.run_job(input, dependency_outputs);
              LodJobOutput::Mesh(Arc::new(lod_chunk_mesh))
            }
          }
        }).unwrap_or_else(|e| panic!("Failed to create job queue: {:?}", e)),
    }
  }

  #[inline]
  pub fn get_max_lod_level(&self) -> u8 { self.max_depth }

  #[profiling::function]
  pub fn update(&mut self, position: Vec3) -> (u32, Isometry3, impl Iterator<Item=(&Aabb, &Arc<E::Chunk>)>) {
    let position = self.transform_inversed.transform_vec(position);

    {
      scope!("Process job queue messages");
      for message in self.job_queue.get_message_receiver().try_iter() {
        match message {
          JobQueueMessage::JobCompleted(job_key, output) => {
            if let (aabb, LodJobOutput::Mesh(arc)) = (job_key, output) {
              self.lod_chunk_meshes.insert(aabb, arc);
              self.requested_meshing.remove(&aabb);
              self.requested_removal.remove(&aabb); // TODO: is this needed?
            }
          }
          JobQueueMessage::PendingJobRemoved(aabb, _) => {
            if aabb.is_user_bit_set() {
              self.requested_removal.remove(&aabb);
            }
          }
          JobQueueMessage::RunningJobRemoved(aabb) => {
            if aabb.is_user_bit_set() {
              self.requested_removal.remove(&aabb);
            }
          }
          JobQueueMessage::CompletedJobRemoved(job_key, output) => {
            if let (aabb, LodJobOutput::Mesh(arc)) = (job_key, output) {
              self.requested_removal.remove(&aabb);
              Self::cache_empty_lod_chunk_mesh(&mut self.empty_lod_chunk_mesh_cache, self.empty_lod_chunk_mesh_cache_size, arc);
            }
          }
          _ => {}
        }
      }
    }

    {
      scope!("Clear active/keep AABBs");
      self.active_aabbs.clear();
      self.prev_keep_aabbs.clear();
      self.keep_aabbs.drain().collect_into(&mut self.prev_keep_aabbs);
    }

    self.update_root_node(position);

    {
      scope!("Process removed AABBs");
      let mut send_error = false;
      for removed in self.prev_keep_aabbs.difference(&self.keep_aabbs) { // OPTO: can we update `prev_keep_aabbs` and then drain it?
        if !self.requested_removal.contains(removed) {
          self.requested_meshing.remove(&removed);
          self.requested_removal.insert(*removed);
          send_error |= self.job_queue.try_remove_job_and_orphaned_dependencies(*removed).is_err();
          if send_error { break; }
          if let Some(arc) = self.lod_chunk_meshes.remove(removed) {
            Self::cache_empty_lod_chunk_mesh(&mut self.empty_lod_chunk_mesh_cache, self.empty_lod_chunk_mesh_cache_size, arc);
          }
        }
      }
      if send_error {
        self.handle_send_error();
      }
    }

    let chunks = self.lod_chunk_meshes.iter().filter(|(aabb, _)| self.active_aabbs.contains(*aabb));
    (self.root_size, self.transform, chunks)
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
    let root = Aabb::root().with_user_bit_set();
    let depth = 0;
    let neighbor_depths = NeighborDepths::default();
    let NodeResult { filled, activated, .. } = self.update_nodes(root, depth, neighbor_depths, position);
    if filled && !activated {
      self.active_aabbs.insert(root);
    }
  }

  #[inline]
  fn update_nodes(&mut self, aabb: Aabb, depth: u8, neighbor_depths: NeighborDepths, position: Vec3) -> NodeResult {
    self.keep_aabbs.insert(aabb);
    let self_filled = self.update_chunk(aabb, neighbor_depths);
    if self.is_terminal(aabb, depth, position) {
      NodeResult::new(self_filled, false, depth)
    } else { // Subdivide
      let mut all_filled = true;
      let mut activated = PerAabbSubdivide::<bool>::with_default();
      let depth_plus_one = depth + 1;
      let mut maximum_depth = depth_plus_one;
      let subdivided @ AabbSubdivide { base, x, y, xy, z, xz, yz, xyz } = aabb.subdivide();

      let xyz_result = {
        let result = self.update_nodes(xyz, depth_plus_one, neighbor_depths, position);
        activated.xyz = result.activated;
        all_filled &= result.filled;
        maximum_depth = maximum_depth.max(result.maximum_depth);
        result
      };
      let yz_result = {
        let mut neighbor_depths = neighbor_depths;
        neighbor_depths.x = xyz_result.maximum_depth;
        let result = self.update_nodes(yz, depth_plus_one, neighbor_depths, position);
        activated.yz = result.activated;
        all_filled &= result.filled;
        maximum_depth = maximum_depth.max(result.maximum_depth);
        result
      };
      let xz_result = {
        let mut neighbor_depths = neighbor_depths;
        neighbor_depths.y = xyz_result.maximum_depth;
        let result = self.update_nodes(xz, depth_plus_one, neighbor_depths, position);
        activated.xz = result.activated;
        all_filled &= result.filled;
        maximum_depth = maximum_depth.max(result.maximum_depth);
        result
      };
      let z_result = {
        let mut neighbor_depths = neighbor_depths;
        neighbor_depths.x = xz_result.maximum_depth;
        neighbor_depths.y = yz_result.maximum_depth;
        neighbor_depths.xy = xyz_result.maximum_depth;
        let result = self.update_nodes(z, depth_plus_one, neighbor_depths, position);
        activated.z = result.activated;
        all_filled &= result.filled;
        maximum_depth = maximum_depth.max(result.maximum_depth);
        result
      };
      let xy_result = {
        let mut neighbor_depths = neighbor_depths;
        neighbor_depths.z = xyz_result.maximum_depth;
        let result = self.update_nodes(xy, depth_plus_one, neighbor_depths, position);
        activated.xy = result.activated;
        all_filled &= result.filled;
        maximum_depth = maximum_depth.max(result.maximum_depth);
        result
      };
      let y_result = {
        let mut neighbor_depths = neighbor_depths;
        neighbor_depths.x = xy_result.maximum_depth;
        neighbor_depths.z = yz_result.maximum_depth;
        neighbor_depths.xz = xyz_result.maximum_depth;
        let result = self.update_nodes(y, depth_plus_one, neighbor_depths, position);
        activated.y = result.activated;
        all_filled &= result.filled;
        maximum_depth = maximum_depth.max(result.maximum_depth);
        result
      };
      let x_result = {
        let mut neighbor_depths = neighbor_depths;
        neighbor_depths.y = xy_result.maximum_depth;
        neighbor_depths.z = xz_result.maximum_depth;
        neighbor_depths.yz = xyz_result.maximum_depth;
        let result = self.update_nodes(x, depth_plus_one, neighbor_depths, position);
        activated.x = result.activated;
        all_filled &= result.filled;
        maximum_depth = maximum_depth.max(result.maximum_depth);
        result
      };
      let _base_result = {
        let mut neighbor_depths = neighbor_depths;
        neighbor_depths.x = x_result.maximum_depth;
        neighbor_depths.y = y_result.maximum_depth;
        neighbor_depths.z = z_result.maximum_depth;
        neighbor_depths.xy = xy_result.maximum_depth;
        neighbor_depths.yz = yz_result.maximum_depth;
        neighbor_depths.xz = xz_result.maximum_depth;
        let result = self.update_nodes(base, depth_plus_one, neighbor_depths, position);
        activated.base = result.activated;
        all_filled &= result.filled;
        maximum_depth = maximum_depth.max(result.maximum_depth);
        result
      };

      if all_filled { // All subdivided nodes are filled, activate each non-activated node.
        for i in 0..8u8 {
          if !activated[i] {
            self.active_aabbs.insert(subdivided[i]);
          }
        }
        NodeResult::new(true, true, maximum_depth) // Act as is filled and activated, because all sub-nodes are filled and activated.
      } else {
        NodeResult::new(self_filled, false, maximum_depth) // Not all subdivided nodes are filled, we might be filled. Our parent should activate us if possible.
      }
    }
  }

  #[inline]
  fn is_terminal(&self, aabb: Aabb, depth: u8, position: Vec3) -> bool {
    if let Some(fixed_lod_level) = self.fixed_lod_level {
      depth >= self.max_depth.min(fixed_lod_level)
    } else {
      depth >= self.max_depth || aabb.distance_from(self.root_size, position) > self.lod_factor * aabb.size(self.root_size) as f32
    }
  }

  fn update_chunk(&mut self, aabb: Aabb, neighbor_depths: NeighborDepths) -> bool {
    if self.lod_chunk_meshes.contains_key(&aabb) { return true; }
    if !self.requested_meshing.contains(&aabb) {
      let empty_lod_chunk_mesh = self.empty_lod_chunk_mesh_cache.pop_front().unwrap_or_else(|| E::Chunk::default());
      let (input, dependencies) = self.extractor.create_job(aabb.with_size(self.root_size), neighbor_depths, self.volume.clone(), empty_lod_chunk_mesh);
      let job = LodJob { aabb, input: LodJobInput::Mesh(input), dependencies: Some(dependencies) };
      self.job_queue.try_add_job(job).unwrap_or_else(|_| self.handle_send_error());
      self.requested_meshing.insert(aabb);
      self.requested_removal.remove(&aabb); // TODO: is this needed?
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
  maximum_depth: u8,
}

impl NodeResult {
  #[inline]
  fn new(filled: bool, activated: bool, maximum_depth: u8) -> Self {
    Self { filled, activated, maximum_depth }
  }
}


// Job types

pub enum LodJobInput<V, JI> {
  Sample(V),
  Mesh(JI),
}

pub struct LodJob<C: ChunkSize, V: Volume, E: LodExtractor<C>> {
  aabb: Aabb,
  input: LodJobInput<V, E::JobInput>,
  dependencies: Option<E::DependenciesIterator<V>>,
}

impl<C: ChunkSize, V: Volume, E: LodExtractor<C>> LodJob<C, V, E> {
  #[inline]
  pub fn new_sample(aabb: Aabb, volume: V) -> Self {
    Self {
      aabb: aabb.with_user_bit_unset(),
      input: LodJobInput::Sample(volume),
      dependencies: None,
    }
  }

  #[inline]
  pub fn new_mesh(aabb: Aabb, extractor_job_input: E::JobInput, extractor_dependencies_iterator: E::DependenciesIterator<V>) -> Self {
    Self {
      aabb: aabb.with_user_bit_set(),
      input: LodJobInput::Mesh(extractor_job_input),
      dependencies: Some(extractor_dependencies_iterator),
    }
  }
}

impl<C: ChunkSize, V: Volume, E: LodExtractor<C>> Job<Aabb, E::DependencyKey, LodJobInput<V, E::JobInput>> for LodJob<C, V, E> {
  #[inline]
  fn key(&self) -> &Aabb { &self.aabb }

  type DependencyIterator = LodJobDependencyIterator<C, V, E>;

  #[inline]
  fn into(self) -> (LodJobInput<V, E::JobInput>, Self::DependencyIterator) {
    let input = self.input;
    let dependencies = LodJobDependencyIterator::<C, V, E>(self.dependencies);
    (input, dependencies)
  }
}

#[repr(transparent)]
pub struct LodJobDependencyIterator<C: ChunkSize, V: Volume, E: LodExtractor<C>>(Option<E::DependenciesIterator<V>>);

impl<C: ChunkSize, V: Volume, E: LodExtractor<C>> Iterator for LodJobDependencyIterator<C, V, E> {
  type Item = (E::DependencyKey, LodJob<C, V, E>);

  #[inline]
  fn next(&mut self) -> Option<Self::Item> {
    match &mut self.0 {
      Some(i) => i.next(),
      _ => None,
    }
  }

  #[inline]
  fn size_hint(&self) -> (usize, Option<usize>) {
    match &self.0 {
      Some(i) => i.size_hint(),
      _ => (0, Some(0)),
    }
  }
}

impl<C: ChunkSize, V: Volume, E: LodExtractor<C>> ExactSizeIterator for LodJobDependencyIterator<C, V, E> where
  E::DependenciesIterator<V>: ExactSizeIterator
{
  fn len(&self) -> usize {
    match &self.0 {
      Some(i) => i.len(),
      None => 0,
    }
  }
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

impl<C: ChunkSize, V: Volume, E: LodExtractor<C>> LodChunkMeshManager<C> for LodOctmap<C, V, E> {
  type Extractor = E;
  #[inline]
  fn get_extractor(&self) -> &E {
    &self.extractor
  }

  #[inline]
  fn update(&mut self, position: Vec3) -> (u32, Isometry3, Box<dyn Iterator<Item=(&Aabb, &Arc<E::Chunk>)> + '_>) {
    let (root_half_size, transform, chunks) = self.update(position);
    (root_half_size, transform, Box::new(chunks))
  }
}

impl<C: ChunkSize, V: Volume, E: LodExtractor<C>> LodChunkMeshManagerParameters for LodOctmap<C, V, E> {
  #[inline]
  fn get_max_lod_level(&self) -> u8 { self.max_depth }

  #[inline]
  fn get_lod_factor(&self) -> f32 { self.lod_factor }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { &mut self.lod_factor }

  #[inline]
  fn get_fixed_lod_level(&self) -> Option<u8> { self.fixed_lod_level }
  #[inline]
  fn get_fixed_lod_level_mut(&mut self) -> &mut Option<u8> { &mut self.fixed_lod_level }
}
