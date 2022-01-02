use criterion::{Criterion, criterion_group, criterion_main};

use voxel_meshing::marching_cubes::{MarchingCubes, MarchingCubesSettings};
use voxel_meshing::octree::{Octree, OctreeSettings};
use voxel_meshing::volume::{Sphere, SphereSettings};

pub fn marching_cubes_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 256.0 });
  let marching_cubes = MarchingCubes::new(MarchingCubesSettings { surface_level: 0.0 });
  let octree = Octree::new(OctreeSettings { total_size: 256, chunk_size: 16 }, sphere, marching_cubes);
  c.bench_function("marching cubes, sphere, LOD 0", |b|b.iter(||octree.generate_into(0, &mut Vec::new())));
  c.bench_function("marching cubes, sphere, LOD 1", |b|b.iter(||octree.generate_into(1, &mut Vec::new())));
}

criterion_group!(benches, marching_cubes_benchmark);
criterion_main!(benches);
