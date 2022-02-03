use criterion::{BatchSize, black_box, Criterion, criterion_group, criterion_main};
use ultraviolet::{UVec3, Vec3};

use voxel_meshing::chunk::{CELLS_IN_CHUNK_USIZE, Chunk};
use voxel_meshing::marching_cubes::MarchingCubes;
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
          black_box(sphere.sample(UVec3::new(x, y, z)));
        }
      }
    }
  }));
}

pub fn marching_cubes_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 64.0 });
  let marching_cubes = MarchingCubes;
  let start = UVec3::new(0, 0, 0);
  let step = 1;
  let samples = sphere.sample_chunk(start, step);
  c.bench_function("Standalone-MC-Sphere", |b| b.iter_batched(
    || Chunk::with_vertices_indices(Vec::with_capacity(CELLS_IN_CHUNK_USIZE), Vec::with_capacity(CELLS_IN_CHUNK_USIZE)), // On average, one triangle per 3 cells. Probably an overestimation, but that is ok.
    |mut chunk| marching_cubes.extract_chunk(start, step, &samples, &mut chunk),
    BatchSize::SmallInput,
  ));
}

pub fn marching_cubes_octree_benchmark(c: &mut Criterion) {
  let total_size = 4096;
  let sphere = Sphere::new(SphereSettings { radius: total_size as f32 });
  let marching_cubes = MarchingCubes;

  let mut group = c.benchmark_group("Octree-MC-Sphere");
  let position = Vec3::zero();
  group.bench_function("4096-1.0", |b| b.iter_batched(
    || preallocate_octree(Octree::new(OctreeSettings { total_size, lod_factor: 1.0, ..OctreeSettings::default() }, sphere, marching_cubes), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-2.0", |b| b.iter_batched(
    || preallocate_octree(Octree::new(OctreeSettings { total_size, lod_factor: 2.0, ..OctreeSettings::default() }, sphere, marching_cubes), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-3.0", |b| b.iter_batched(
    || preallocate_octree(Octree::new(OctreeSettings { total_size, lod_factor: 3.0, ..OctreeSettings::default() }, sphere, marching_cubes), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-4.0", |b| b.iter_batched(
    || preallocate_octree(Octree::new(OctreeSettings { total_size, lod_factor: 4.0, ..OctreeSettings::default() }, sphere, marching_cubes), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.finish();
}

fn preallocate_octree<V: Volume + Clone + Send + 'static>(mut octree: Octree<V>, position: Vec3) -> Octree<V> {
  drop(octree.update(position));
  octree.clear();
  octree
}

criterion_group!(benches, sphere_benchmark, marching_cubes_benchmark, marching_cubes_octree_benchmark);
criterion_main!(benches);
