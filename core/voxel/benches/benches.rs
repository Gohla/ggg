#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use criterion::{BatchSize, black_box, Criterion, criterion_group, criterion_main};
use ultraviolet::{Isometry3, UVec3, Vec3};

use voxel::chunk::{ChunkSize, ChunkSize16, ChunkMesh};
use voxel::marching_cubes::MarchingCubes;
use voxel::lod_volume::{AABB, LodOctmap, LodOctmapSettings};
use voxel::transvoxel::side::TransitionSide;
use voxel::transvoxel::Transvoxel;
use voxel::volume::{Sphere, SphereSettings, Volume};

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

type C = ChunkSize16;

pub fn marching_cubes_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 16.0 });
  let marching_cubes = MarchingCubes::<C>::new();
  let start = UVec3::new(0, 0, 0);
  let step = 1;
  let samples = sphere.sample_chunk(start, step);
  c.bench_function("Standalone-MarchingCubes-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C>(),
    |mut chunk| marching_cubes.extract_chunk(start, step, &samples, &mut chunk),
    BatchSize::SmallInput,
  ));
}

pub fn transvoxel_benchmark(c: &mut Criterion) {
  let size = 64;
  let sphere = Sphere::new(SphereSettings { radius: size as f32 });
  let transvoxel = Transvoxel::<C>::new();

  let aabb = AABB::from_size(size);
  let aabbs = aabb.subdivide();
  let lores_aabb = aabbs[4]; // 4th subdivision is at 0,0 with z as center.
  let lores_min = lores_aabb.min();
  let lores_step = lores_aabb.step::<C>();

  let side = TransitionSide::LoZ;
  let hires_step = 1;
  let hires_chunk_mins = side.subdivided_face_of_side_minimums(lores_aabb);
  let hires_chunk_samples = [
    sphere.sample_chunk(hires_chunk_mins[0], hires_step),
    sphere.sample_chunk(hires_chunk_mins[1], hires_step),
    sphere.sample_chunk(hires_chunk_mins[2], hires_step),
    sphere.sample_chunk(hires_chunk_mins[3], hires_step),
  ];
  c.bench_function("Standalone-Transvoxel-LoZ-Sphere-64", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C>(),
    |mut chunk| transvoxel.extract_chunk(side, &hires_chunk_mins, &hires_chunk_samples, hires_step, lores_min, lores_step, &mut chunk),
    BatchSize::SmallInput,
  ));
}

fn preallocate_chunk_vertices<C: ChunkSize>() -> ChunkMesh {
  // On average, one triangle per 3 cells. Probably an overestimation, but that is ok.
  ChunkMesh::with_vertices_indices(Vec::with_capacity(C::CELLS_IN_CHUNK_USIZE), Vec::with_capacity(C::CELLS_IN_CHUNK_USIZE))
}

pub fn octree_benchmark(c: &mut Criterion) {
  let total_size = 4096;
  let transform = Isometry3::identity();
  let sphere = Sphere::new(SphereSettings { radius: total_size as f32 });
  let marching_cubes = MarchingCubes::<C>::new();
  let transvoxel = Transvoxel::<C>::new();

  let mut group = c.benchmark_group("Octree-Sphere");
  let position = Vec3::zero();
  group.bench_function("4096-1.0", |b| b.iter_batched(
    || preallocate_octree(LodOctmap::new(LodOctmapSettings { total_size, lod_factor: 1.0, ..LodOctmapSettings::default() }, transform, sphere, marching_cubes, transvoxel), position),
    |mut octree| drop(black_box(octree.do_update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-2.0", |b| b.iter_batched(
    || preallocate_octree(LodOctmap::new(LodOctmapSettings { total_size, lod_factor: 2.0, ..LodOctmapSettings::default() }, transform, sphere, marching_cubes, transvoxel), position),
    |mut octree| drop(black_box(octree.do_update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-3.0", |b| b.iter_batched(
    || preallocate_octree(LodOctmap::new(LodOctmapSettings { total_size, lod_factor: 3.0, ..LodOctmapSettings::default() }, transform, sphere, marching_cubes, transvoxel), position),
    |mut octree| drop(black_box(octree.do_update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-4.0", |b| b.iter_batched(
    || preallocate_octree(LodOctmap::new(LodOctmapSettings { total_size, lod_factor: 4.0, ..LodOctmapSettings::default() }, transform, sphere, marching_cubes, transvoxel), position),
    |mut octree| drop(black_box(octree.do_update(position))),
    BatchSize::SmallInput,
  ));
  group.finish();
}

fn preallocate_octree<V: Volume + Clone + Send + 'static, C: ChunkSize>(mut octree: LodOctmap<V, C>, position: Vec3) -> LodOctmap<V, C> where
  [f32; C::VOXELS_IN_CHUNK_USIZE]:,
  [u16; MarchingCubes::<C>::SHARED_INDICES_SIZE]:,
  [u16; Transvoxel::<C>::SHARED_INDICES_SIZE]:,
{
  drop(octree.do_update(position));
  octree.clear();
  octree
}

criterion_group!(benches, sphere_benchmark, marching_cubes_benchmark, transvoxel_benchmark, octree_benchmark);
criterion_main!(benches);

