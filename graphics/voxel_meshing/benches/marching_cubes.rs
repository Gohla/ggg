use criterion::{BatchSize, BenchmarkId, black_box, Criterion, criterion_group, criterion_main};
use ultraviolet::UVec3;

use voxel_meshing::marching_cubes::{MarchingCubes, MarchingCubesSettings};
use voxel_meshing::octree::{Octree, OctreeSettings};
use voxel_meshing::volume::{Sphere, SphereSettings, Volume};

pub fn sphere_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 64.0 });
  let start = UVec3::new(0, 0, 0);
  let end = UVec3::new(64, 64, 64);
  c.bench_function("Standalone-Sphere-64", |b| b.iter(|| {
    for x in start.x..=end.x {
      for y in start.y..=end.y {
        for z in start.z..=end.z {
          black_box(sphere.sample(&UVec3::new(x, y, z)));
        }
      }
    }
  }));
}

pub fn marching_cubes_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 64.0 });
  let marching_cubes = MarchingCubes::new(MarchingCubesSettings { surface_level: 0.0 });
  let start = UVec3::new(0, 0, 0);
  let end = UVec3::new(64, 64, 64);
  let step = 1;
  c.bench_function("Standalone-MC-Sphere-64", |b| b.iter_batched(
    || Vec::with_capacity(64 * 64 * 64), // On average, one triangle per 3 cells. Probably an overestimation, but that is ok.
    |mut vertices| marching_cubes.generate_into(start, end, step, &sphere, &mut vertices),
    BatchSize::SmallInput,
  ));
}

pub fn marching_cubes_octree_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 256.0 });
  let marching_cubes = MarchingCubes::new(MarchingCubesSettings { surface_level: 0.0 });

  let mut group = c.benchmark_group("Octree-MC-Sphere");
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

criterion_group!(benches, sphere_benchmark, marching_cubes_benchmark, marching_cubes_octree_benchmark);
criterion_main!(benches);
