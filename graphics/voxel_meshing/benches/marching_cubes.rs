use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use voxel_meshing::marching_cubes::{MarchingCubes, MarchingCubesSettings};
use voxel_meshing::octree::{Octree, OctreeSettings};
use voxel_meshing::volume::{Sphere, SphereSettings};

pub fn marching_cubes_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 256.0 });
  let marching_cubes = MarchingCubes::new(MarchingCubesSettings { surface_level: 0.0 });

  let mut group = c.benchmark_group("MC-Sphere");
  let octree = Octree::new(OctreeSettings { total_size: 256, chunk_size: 16 }, sphere, marching_cubes);
  group.bench_with_input(BenchmarkId::from_parameter("256-16-0"), &octree, |b, o| b.iter(|| o.generate_into(0, &mut Vec::new())));
  group.bench_with_input(BenchmarkId::from_parameter("256-16-1"), &octree, |b, o| b.iter(|| o.generate_into(1, &mut Vec::new())));
  let octree = Octree::new(OctreeSettings { total_size: 256, chunk_size: 4 }, sphere, marching_cubes);
  group.bench_with_input(BenchmarkId::from_parameter("256-4-0"), &octree, |b, o| b.iter(|| o.generate_into(0, &mut Vec::new())));
  group.bench_with_input(BenchmarkId::from_parameter("256-4-1"), &octree, |b, o| b.iter(|| o.generate_into(1, &mut Vec::new())));
  group.bench_with_input(BenchmarkId::from_parameter("256-4-2"), &octree, |b, o| b.iter(|| o.generate_into(2, &mut Vec::new())));
  group.bench_with_input(BenchmarkId::from_parameter("256-4-3"), &octree, |b, o| b.iter(|| o.generate_into(3, &mut Vec::new())));
  let octree = Octree::new(OctreeSettings { total_size: 256, chunk_size: 1 }, sphere, marching_cubes);
  group.bench_with_input(BenchmarkId::from_parameter("256-1-0"), &octree, |b, o| b.iter(|| o.generate_into(0, &mut Vec::new())));
  group.bench_with_input(BenchmarkId::from_parameter("256-1-1"), &octree, |b, o| b.iter(|| o.generate_into(1, &mut Vec::new())));
  group.bench_with_input(BenchmarkId::from_parameter("256-1-2"), &octree, |b, o| b.iter(|| o.generate_into(2, &mut Vec::new())));
  group.bench_with_input(BenchmarkId::from_parameter("256-1-3"), &octree, |b, o| b.iter(|| o.generate_into(3, &mut Vec::new())));
  group.bench_with_input(BenchmarkId::from_parameter("256-1-4"), &octree, |b, o| b.iter(|| o.generate_into(4, &mut Vec::new())));
  group.finish();
}

criterion_group!(benches, marching_cubes_benchmark);
criterion_main!(benches);
