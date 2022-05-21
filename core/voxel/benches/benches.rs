use criterion::{BatchSize, black_box, Criterion, criterion_group, criterion_main};
use ultraviolet::{Isometry3, UVec3, Vec3};

use voxel::chunk::mesh::ChunkMesh;
use voxel::chunk::size::{ChunkSize, ChunkSize16};
use voxel::lod::aabb::AABB;
use voxel::lod::extract::LodExtractor;
use voxel::lod::octmap::{LodOctmap, LodOctmapSettings};
use voxel::lod::transvoxel::TransvoxelExtractor;
use voxel::marching_cubes::MarchingCubes;
use voxel::surface_nets::SurfaceNets;
use voxel::transvoxel::side::TransitionSide;
use voxel::transvoxel::Transvoxel;
use voxel::volume::{Sphere, SphereSettings, Volume};

pub fn sphere_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 16.0 });
  let start = UVec3::new(0, 0, 0);
  let step = 1;
  c.bench_function("Volume-Sphere-Sample-16", |b| b.iter(|| {
    for z in start.z..ChunkSize16::VOXELS_IN_CHUNK_ROW {
      for y in start.y..ChunkSize16::VOXELS_IN_CHUNK_ROW {
        for x in start.x..ChunkSize16::VOXELS_IN_CHUNK_ROW {
          black_box(sphere.sample(UVec3::new(x, y, z)));
        }
      }
    }
  }));
  c.bench_function("Volume-Sphere-Sample-Chunk-16", |b| b.iter(|| {
    black_box(sphere.sample_chunk::<ChunkSize16>(start, step));
  }));
}

type C16 = ChunkSize16;

pub fn marching_cubes_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 16.0 });
  let marching_cubes = MarchingCubes::<C16>::new();
  let start = UVec3::new(0, 0, 0);
  let step = 1;
  let chunk_samples = sphere.sample_chunk(start, step);
  c.bench_function("MarchingCubes-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| marching_cubes.extract_chunk(start, step, &chunk_samples, &mut chunk_mesh),
    BatchSize::SmallInput,
  ));
}

pub fn transvoxel_benchmark(c: &mut Criterion) {
  let size = 64;
  let sphere = Sphere::new(SphereSettings { radius: size as f32 });
  let transvoxel = Transvoxel::<C16>::new();

  let aabb = AABB::from_size(size);
  let aabbs = aabb.subdivide();
  let lores_aabb = aabbs[4]; // 4th subdivision is at 0,0 with z as center.
  let lores_min = lores_aabb.min();
  let lores_step = lores_aabb.step::<C16>();

  let side = TransitionSide::LoZ;
  let hires_step = 1;
  let hires_chunk_mins = side.subdivided_face_of_side_minimums(lores_aabb);
  let hires_chunk_samples = [
    sphere.sample_chunk(hires_chunk_mins[0], hires_step),
    sphere.sample_chunk(hires_chunk_mins[1], hires_step),
    sphere.sample_chunk(hires_chunk_mins[2], hires_step),
    sphere.sample_chunk(hires_chunk_mins[3], hires_step),
  ];
  c.bench_function("Transvoxel-LoZ-Sphere-64", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| transvoxel.extract_chunk(side, &hires_chunk_mins, &hires_chunk_samples, hires_step, lores_min, lores_step, &mut chunk_mesh),
    BatchSize::SmallInput,
  ));
}

pub fn surface_nets_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 16.0 });
  let surface_nets = SurfaceNets::<C16>::new();
  let start = UVec3::new(0, 0, 0);
  let step = 1;
  let chunk_samples = sphere.sample_chunk(start, step);
  c.bench_function("SurfaceNets-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| surface_nets.extract_chunk(start, step, &chunk_samples, &mut chunk_mesh),
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
  let volume = Sphere::new(SphereSettings { radius: total_size as f32 });
  let extractor = TransvoxelExtractor::default();

  let mut group = c.benchmark_group("Octree-Sphere-Transvoxel");
  let position = Vec3::zero();
  group.bench_function("4096-1.0", |b| b.iter_batched(
    || preallocate_octmap::<_, C16, _>(LodOctmap::new(LodOctmapSettings { total_size, lod_factor: 1.0, ..LodOctmapSettings::default() }, transform, volume, extractor), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-2.0", |b| b.iter_batched(
    || preallocate_octmap(LodOctmap::new(LodOctmapSettings { total_size, lod_factor: 2.0, ..LodOctmapSettings::default() }, transform, volume, extractor), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-3.0", |b| b.iter_batched(
    || preallocate_octmap(LodOctmap::new(LodOctmapSettings { total_size, lod_factor: 3.0, ..LodOctmapSettings::default() }, transform, volume, extractor), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-4.0", |b| b.iter_batched(
    || preallocate_octmap(LodOctmap::new(LodOctmapSettings { total_size, lod_factor: 4.0, ..LodOctmapSettings::default() }, transform, volume, extractor), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.finish();
}

fn preallocate_octmap<V: Volume, C: ChunkSize, E: LodExtractor<C>>(mut octree: LodOctmap<V, C, E>, position: Vec3) -> LodOctmap<V, C, E> {
  drop(octree.update(position));
  octree.clear();
  octree
}

criterion_group!(benches, sphere_benchmark, marching_cubes_benchmark, transvoxel_benchmark, surface_nets_benchmark, octree_benchmark);
criterion_main!(benches);
