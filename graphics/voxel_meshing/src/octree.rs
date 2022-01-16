#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use tracing::debug;
use ultraviolet::{UVec3, Vec3};

use crate::marching_cubes::{CHUNK_SIZE, MarchingCubes};
use crate::vertex::Vertex;
use crate::volume::Volume;

// Trait

pub trait VolumeMeshManager {
  fn get_max_lod_level(&self) -> u32;
  fn get_lod_factor(&self) -> f32;
  fn get_lod_factor_mut(&mut self) -> &mut f32;

  fn update(&mut self, position: Vec3) -> Box<dyn Iterator<Item=(&AABB, &(Vec<Vertex>, bool))> + '_>;
}

// Octree settings


#[derive(Copy, Clone, Debug)]
pub struct OctreeSettings {
  pub total_size: u32,
  pub lod_factor: f32,
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
    }
  }
}

// Octree

pub struct Octree<V: Volume> {
  total_size: u32,
  lod_factor: f32,

  max_lod_level: u32,
  volume: V,
  marching_cubes: MarchingCubes,

  active_aabbs: HashSet<AABB>,
  meshes: HashMap<AABB, (Vec<Vertex>, bool)>,
}

impl<V: Volume> Octree<V> {
  pub fn new(settings: OctreeSettings, volume: V, marching_cubes: MarchingCubes) -> Self {
    settings.check();
    let lod_0_step = settings.total_size / CHUNK_SIZE;
    let max_lod_level = lod_0_step.log2();
    Self {
      total_size: settings.total_size,
      lod_factor: settings.lod_factor,
      max_lod_level,
      volume,
      marching_cubes,
      active_aabbs: HashSet::new(),
      meshes: HashMap::new(),
    }
  }

  #[inline]
  pub fn get_max_lod_level(&self) -> u32 { self.max_lod_level }

  pub fn update(&mut self, position: Vec3) -> impl Iterator<Item=(&AABB, &(Vec<Vertex>, bool))> {
    let prev_active: HashSet<_> = self.active_aabbs.drain().collect();
    self.update_nodes(AABB::from_size(self.total_size), 0, position);
    for removed in prev_active.difference(&self.active_aabbs) {
      if let Some((mesh, filled)) = self.meshes.get_mut(removed) {
        debug!("Removing unused mesh for {:?}", removed);
        mesh.clear();
        *filled = false;
      }
    }
    self.meshes.iter().filter(|(aabb, _)| self.active_aabbs.contains(*aabb))
  }

  pub fn clear(&mut self) {
    self.active_aabbs.clear();
    for (_, (vertices, filled)) in &mut self.meshes {
      vertices.clear();
      *filled = false;
    }
  }


  fn update_nodes(&mut self, aabb: AABB, lod_level: u32, position: Vec3) {
    if self.is_terminal(aabb, lod_level, position) {
      self.active_aabbs.insert(aabb);
      self.generate_mesh(aabb);
    } else { // Subdivide
      for sub_aabb in aabb.subdivide() {
        self.update_nodes(sub_aabb, lod_level + 1, position);
      }
    }
  }

  #[inline]
  fn is_terminal(&self, aabb: AABB, lod_level: u32, position: Vec3) -> bool {
    lod_level >= self.max_lod_level || aabb.distance_from(position) > self.lod_factor * aabb.size() as f32
  }

  #[inline]
  fn generate_mesh(&mut self, aabb: AABB) {
    let (vertices, filled) = self.meshes.entry(aabb).or_default();
    if *filled { return; }
    vertices.clear();
    let step = aabb.size() / CHUNK_SIZE;
    debug!("Running MC for {:?} step {}", aabb, step);
    self.marching_cubes.generate_into(aabb.min, step, &self.volume, vertices);
    *filled = true;
  }
}

impl<V: Volume> VolumeMeshManager for Octree<V> {
  #[inline]
  fn get_max_lod_level(&self) -> u32 { self.max_lod_level }
  #[inline]
  fn get_lod_factor(&self) -> f32 { self.lod_factor }
  #[inline]
  fn get_lod_factor_mut(&mut self) -> &mut f32 { &mut self.lod_factor }

  #[inline]
  fn update(&mut self, position: Vec3) -> Box<dyn Iterator<Item=(&AABB, &(Vec<Vertex>, bool))> + '_> { Box::new(self.update(position)) }
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


  #[inline(always)]
  fn new_unchecked(min: UVec3, size: u32) -> Self {
    Self { min, size }
  }
}
