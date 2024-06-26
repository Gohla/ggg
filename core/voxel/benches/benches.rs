use criterion::{BatchSize, black_box, Criterion, criterion_group, criterion_main};
use ultraviolet::{Isometry3, UVec3, Vec3};

use voxel::chunk::mesh::ChunkMesh;
use voxel::chunk::size::{ChunkSize, ChunkSize16, ChunkSize32};
use voxel::lod::aabb::Aabb;
use voxel::lod::extract::LodExtractor;
use voxel::lod::octmap::{LodOctmap, LodOctmapSettings};
use voxel::lod::transvoxel::TransvoxelExtractor;
use voxel::marching_cubes::MarchingCubes;
use voxel::surface_nets::lod::SurfaceNetsLod;
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
type C32 = ChunkSize32;

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
  let root_size = 64;
  let sphere = Sphere::new(SphereSettings { radius: root_size as f32 });
  let transvoxel = Transvoxel::<C16>::new();

  let aabb = Aabb::root();
  let aabbs = aabb.subdivide_array();
  let lores_aabb = aabbs[4]; // 4th subdivision is at 0,0 with z as center.
  let lores_min = lores_aabb.minimum_point(root_size);
  let lores_step = lores_aabb.step::<C16>(root_size);

  let side = TransitionSide::LoZ;
  let hires_step = 1;
  let hires_chunk_mins = side.subdivided_face_of_side_minimums(lores_aabb.with_size(root_size));
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
  let step = 1;
  let start = UVec3::new(0, 0, 0);
  {
    let sphere = Sphere::new(SphereSettings { radius: 16.0 });
    let surface_nets = SurfaceNets::<C16>::new();
    let chunk_samples = sphere.sample_chunk(start, step);
    c.bench_function("SurfaceNets-Sphere-16", |b| b.iter_batched(
      || preallocate_chunk_vertices::<C16>(),
      |mut chunk_mesh| surface_nets.extract_chunk_from_maybe_compressed_samples(start, step, &chunk_samples, &mut chunk_mesh),
      BatchSize::SmallInput,
    ));
  }
  {
    let sphere = Sphere::new(SphereSettings { radius: 32.0 });
    let surface_nets = SurfaceNets::<C32>::new();
    let chunk_samples = sphere.sample_chunk(start, step);
    c.bench_function("SurfaceNets-Sphere-32", |b| b.iter_batched(
      || preallocate_chunk_vertices::<C32>(),
      |mut chunk_mesh| surface_nets.extract_chunk_from_maybe_compressed_samples(start, step, &chunk_samples, &mut chunk_mesh),
      BatchSize::SmallInput,
    ));
  }
}

pub fn surface_nets_borders_benchmark(c: &mut Criterion) {
  let sphere = Sphere::new(SphereSettings { radius: 32.0 });
  let surface_nets_lod = SurfaceNetsLod::<C16>::new();
  let step = 1;
  let min = UVec3::new(0, 0, 0);
  let chunk_samples = sphere.sample_chunk(min, step);
  let min_x = UVec3::new(16, 0, 0);
  let chunk_samples_x = sphere.sample_chunk(min_x, step);
  let min_y = UVec3::new(0, 16, 0);
  let chunk_samples_y = sphere.sample_chunk(min_y, step);
  let min_z = UVec3::new(0, 0, 16);
  let chunk_samples_z = sphere.sample_chunk(min_z, step);
  let min_xy = UVec3::new(16, 16, 0);
  let chunk_samples_xy = sphere.sample_chunk(min_xy, step);
  let min_yz = UVec3::new(0, 16, 16);
  let chunk_samples_yz = sphere.sample_chunk(min_yz, step);
  let min_xz = UVec3::new(16, 0, 16);
  let chunk_samples_xz = sphere.sample_chunk(min_xz, step);
  c.bench_function("SurfaceNets-Border-X-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| surface_nets_lod.extract_border_x(step, min, &chunk_samples, min_x, &chunk_samples_x, &mut chunk_mesh),
    BatchSize::SmallInput,
  ));
  c.bench_function("SurfaceNets-Border-Y-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| surface_nets_lod.extract_border_y(step, min, &chunk_samples, min_y, &chunk_samples_y, &mut chunk_mesh),
    BatchSize::SmallInput,
  ));
  c.bench_function("SurfaceNets-Border-Z-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| surface_nets_lod.extract_border_z(step, min, &chunk_samples, min_z, &chunk_samples_z, &mut chunk_mesh),
    BatchSize::SmallInput,
  ));
  c.bench_function("SurfaceNets-Border-XY-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| surface_nets_lod.extract_border_xy(step, min, &chunk_samples, min_x, &chunk_samples_x, min_y, &chunk_samples_y, min_xy, &chunk_samples_xy, &mut chunk_mesh),
    BatchSize::SmallInput,
  ));
  c.bench_function("SurfaceNets-Border-YZ-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| surface_nets_lod.extract_border_yz(step, min, &chunk_samples, min_y, &chunk_samples_y, min_z, &chunk_samples_z, min_yz, &chunk_samples_yz, &mut chunk_mesh),
    BatchSize::SmallInput,
  ));
  c.bench_function("SurfaceNets-Border-XZ-Sphere-16", |b| b.iter_batched(
    || preallocate_chunk_vertices::<C16>(),
    |mut chunk_mesh| surface_nets_lod.extract_border_xz(step, min, &chunk_samples, min_x, &chunk_samples_x, min_z, &chunk_samples_z, min_xz, &chunk_samples_xz, &mut chunk_mesh),
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
    || preallocate_octmap::<C16, _, _>(LodOctmap::new(LodOctmapSettings { root_size: total_size, lod_factor: 1.0, ..LodOctmapSettings::default() }, transform, volume, extractor), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-2.0", |b| b.iter_batched(
    || preallocate_octmap(LodOctmap::new(LodOctmapSettings { root_size: total_size, lod_factor: 2.0, ..LodOctmapSettings::default() }, transform, volume, extractor), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-3.0", |b| b.iter_batched(
    || preallocate_octmap(LodOctmap::new(LodOctmapSettings { root_size: total_size, lod_factor: 3.0, ..LodOctmapSettings::default() }, transform, volume, extractor), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.bench_function("4096-4.0", |b| b.iter_batched(
    || preallocate_octmap(LodOctmap::new(LodOctmapSettings { root_size: total_size, lod_factor: 4.0, ..LodOctmapSettings::default() }, transform, volume, extractor), position),
    |mut octree| drop(black_box(octree.update(position))),
    BatchSize::SmallInput,
  ));
  group.finish();
}

fn preallocate_octmap<C: ChunkSize, V: Volume, E: LodExtractor<C>>(mut octree: LodOctmap<C, V, E>, position: Vec3) -> LodOctmap<C, V, E> {
  drop(octree.update(position));
  octree.clear();
  octree
}

criterion_group!(benches, sphere_benchmark, marching_cubes_benchmark, transvoxel_benchmark, surface_nets_benchmark, surface_nets_borders_benchmark, octree_benchmark);
criterion_main!(benches);
