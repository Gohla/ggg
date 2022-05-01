use egui::{ComboBox, Ui};
use egui::color_picker::Alpha;
use ultraviolet::Isometry3;

use gui_widget::UiWidgetsExt;
use voxel::chunk::GenericChunkSize;
use voxel::lod::builder::LodManagerBuilder;
use voxel::lod::chunk_mesh::LodChunkMeshManagerParameters;
use voxel::lod::marching_cubes::{MarchingCubesExtractor, MarchingCubesExtractorSettings};
use voxel::lod::octmap::LodOctmapSettings;
use voxel::lod::render::{LodRenderData, LodRenderDataManager, LodRenderDataSettings};
use voxel::lod::surface_nets::{SurfaceNetsExtractor, SurfaceNetsExtractorSettings};
use voxel::lod::transvoxel::{TransvoxelExtractor, TransvoxelExtractorSettings};
use voxel::marching_cubes::MarchingCubes;
use voxel::surface_nets::SurfaceNets;
use voxel::transvoxel::Transvoxel;
use voxel::uniform::LightSettings;
use voxel::volume::{Noise, NoiseSettings, Plus, Sphere, SphereSettings, Volume};

use crate::stars::StarsRendererSettings;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum VolumeType {
  Sphere,
  Noise,
  SpherePlusNoise,
}

impl Default for VolumeType {
  fn default() -> Self { Self::Sphere }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum ExtractorType {
  MarchingCubes,
  Transvoxel,
  SurfaceNets,
}

impl Default for ExtractorType {
  fn default() -> Self { Self::MarchingCubes }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Settings {
  pub light: LightSettings,

  pub volume_type: VolumeType,
  pub sphere_settings: SphereSettings,
  pub noise_settings: NoiseSettings,

  pub extractor_type: ExtractorType,
  pub marching_cubes_settings: MarchingCubesExtractorSettings,
  pub transvoxel_settings: TransvoxelExtractorSettings,
  pub surface_nets_settings: SurfaceNetsExtractorSettings,

  pub lod_octmap_settings: LodOctmapSettings,
  pub lod_octmap_transform: Isometry3,

  pub lod_render_data_settings: LodRenderDataSettings,
  pub auto_update: bool,

  pub stars_renderer_settings: StarsRendererSettings,
}

type C16 = GenericChunkSize<16>;

impl Settings {
  pub fn create_lod_render_data_manager(
    &self,
  ) -> Box<dyn LodRenderDataManager<C16>> {
    let builder = LodManagerBuilder::new::<C16>();
    match self.volume_type {
      VolumeType::Sphere => self.build_lod_render_data_manager(builder.with_volume(Sphere::new(self.sphere_settings))),
      VolumeType::Noise => self.build_lod_render_data_manager(builder.with_volume(Noise::new(self.noise_settings))),
      VolumeType::SpherePlusNoise => self.build_lod_render_data_manager(builder.with_volume(Plus::new(Sphere::new(self.sphere_settings), Noise::new(self.noise_settings)))),
    }
  }

  fn build_lod_render_data_manager<V: Volume, E>(
    &self,
    builder: LodManagerBuilder<C16, V, E>,
  ) -> Box<dyn LodRenderDataManager<C16>> {
    match self.extractor_type {
      ExtractorType::MarchingCubes => builder
        .with_extractor(MarchingCubesExtractor::new(MarchingCubes::<C16>::default(), self.marching_cubes_settings))
        .build_boxed(self.lod_octmap_settings, self.lod_octmap_transform),
      ExtractorType::Transvoxel => builder
        .with_extractor(TransvoxelExtractor::new(MarchingCubes::<C16>::default(), Transvoxel::<C16>::default(), self.transvoxel_settings))
        .build_boxed(self.lod_octmap_settings, self.lod_octmap_transform),
      ExtractorType::SurfaceNets => builder
        .with_extractor(SurfaceNetsExtractor::new(SurfaceNets::<C16>::default(), self.surface_nets_settings))
        .build_boxed(self.lod_octmap_settings, self.lod_octmap_transform),
    }
  }

  pub fn draw_light_gui(&mut self, ui: &mut Ui) {
    self.light.render_gui(ui);
  }

  /// Returns true if update button was pressed.
  pub fn draw_volume_gui(&mut self, ui: &mut Ui) -> bool {
    ui.collapsing_open_with_grid("Volume", "Grid", |ui| {
      ui.label("Type");
      ComboBox::from_id_source("Type")
        .selected_text(format!("{:?}", self.volume_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.volume_type, VolumeType::Sphere, "Sphere");
          ui.selectable_value(&mut self.volume_type, VolumeType::Noise, "Noise");
          ui.selectable_value(&mut self.volume_type, VolumeType::SpherePlusNoise, "Sphere + Noise");
        });
      ui.end_row();
      match self.volume_type {
        VolumeType::Sphere => self.draw_sphere_settings(ui),
        VolumeType::Noise => self.draw_noise_settings(ui),
        VolumeType::SpherePlusNoise => {
          self.draw_sphere_settings(ui);
          self.draw_noise_settings(ui);
        }
      }
      return ui.button("Update").clicked();
    }).body_returned.map(|i| i.inner).unwrap_or(false)
  }

  fn draw_sphere_settings(&mut self, ui: &mut Ui) {
    ui.label("Radius");
    ui.drag_unlabelled(&mut self.sphere_settings.radius, 0.1);
    ui.end_row();
  }

  fn draw_noise_settings(&mut self, ui: &mut Ui) {
    ui.label("Seed");
    ui.drag_unlabelled(&mut self.noise_settings.seed, 1);
    ui.end_row();
    ui.label("Lacunarity");
    ui.drag_unlabelled_range(&mut self.noise_settings.lacunarity, 0.01, 0.0..=10.0);
    ui.end_row();
    ui.label("Frequency");
    ui.drag_unlabelled_range(&mut self.noise_settings.frequency, 0.001, 0.0..=1.0);
    ui.end_row();
    ui.label("Gain");
    ui.drag_unlabelled_range(&mut self.noise_settings.gain, 0.01, 0.0..=10.0);
    ui.end_row();
    ui.label("Octaves");
    ui.drag_unlabelled_range(&mut self.noise_settings.octaves, 1, 0..=7);
    ui.end_row();
  }

  /// Returns true if update button was pressed.
  pub fn draw_extractor_gui(&mut self, ui: &mut Ui) -> bool {
    ui.collapsing_open_with_grid("Extractor", "Grid", |ui| {
      ui.label("Type");
      ComboBox::from_id_source("Type")
        .selected_text(format!("{:?}", self.extractor_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.extractor_type, ExtractorType::MarchingCubes, "Marching Cubes");
          ui.selectable_value(&mut self.extractor_type, ExtractorType::Transvoxel, "Transvoxel");
          ui.selectable_value(&mut self.extractor_type, ExtractorType::SurfaceNets, "Surface Nets");
        });
      ui.end_row();
      match self.extractor_type {
        ExtractorType::MarchingCubes => {}
        ExtractorType::Transvoxel => {
          ui.label("Render regular chunks?");
          ui.checkbox(&mut self.transvoxel_settings.extract_regular_chunks, "");
          ui.end_row();
          ui.label("extract transition chunks?");
          ui.grid("Transition cell extracting", |ui| {
            ui.checkbox(&mut self.transvoxel_settings.extract_transition_lo_x_chunks, "Lo X");
            ui.checkbox(&mut self.transvoxel_settings.extract_transition_hi_x_chunks, "Hi X");
            ui.end_row();
            ui.checkbox(&mut self.transvoxel_settings.extract_transition_lo_y_chunks, "Lo Y");
            ui.checkbox(&mut self.transvoxel_settings.extract_transition_hi_y_chunks, "Hi Y");
            ui.end_row();
            ui.checkbox(&mut self.transvoxel_settings.extract_transition_lo_z_chunks, "Lo Z");
            ui.checkbox(&mut self.transvoxel_settings.extract_transition_hi_z_chunks, "Hi Z");
            ui.end_row();
          });
        }
        ExtractorType::SurfaceNets => {}
      }
      return ui.button("Update").clicked();
    }).body_returned.map(|i| i.inner).unwrap_or(false)
  }

  pub fn draw_lod_chunk_vertices_manager_gui(
    &mut self,
    ui: &mut Ui,
    lod_chunk_mesh_manager: &mut dyn LodChunkMeshManagerParameters,
  ) {
    ui.collapsing_open_with_grid("LOD chunk mesh manager", "Grid", |ui| {
      ui.label("LOD factor");
      ui.drag_unlabelled_range(lod_chunk_mesh_manager.get_lod_factor_mut(), 0.1, 0.0..=4.0);
      ui.end_row();
      ui.label("Max LOD level");
      ui.monospace(format!("{}", lod_chunk_mesh_manager.get_max_lod_level()));
      ui.end_row();
    });
  }

  /// Returns true if update button was pressed or auto update is set to true.
  pub fn draw_lod_render_data_manager_gui(
    &mut self,
    ui: &mut Ui,
  ) -> bool {
    ui.collapsing_open_with_grid("LOD render data manager", "Grid", |ui| {
      ui.label("Debug render vertices?");
      ui.horizontal(|ui| {
        ui.checkbox(&mut self.lod_render_data_settings.debug_render_vertices, "");
        ui.drag_unlabelled_range(&mut self.lod_render_data_settings.debug_render_vertex_point_size, 0.1, 0.0..=10.0);
        ui.edit_color_vec4(&mut self.lod_render_data_settings.debug_render_vertex_color, Alpha::OnlyBlend);
      });
      ui.end_row();
      ui.label("Debug render edges?");
      ui.horizontal(|ui| {
        ui.checkbox(&mut self.lod_render_data_settings.debug_render_edges, "");
        ui.edit_color_vec4(&mut self.lod_render_data_settings.debug_render_edge_color, Alpha::OnlyBlend);
      });
      ui.end_row();
      ui.label("Debug render octree nodes?");
      ui.horizontal(|ui| {
        ui.checkbox(&mut self.lod_render_data_settings.debug_render_octree_nodes, "");
        ui.edit_color_vec4(&mut self.lod_render_data_settings.debug_render_octree_node_color, Alpha::OnlyBlend);
        ui.edit_color_vec4(&mut self.lod_render_data_settings.debug_render_octree_node_empty_color, Alpha::OnlyBlend);
      });
      ui.end_row();
      let mut update = false;
      if ui.button("Update").clicked() {
        update = true;
      }
      ui.checkbox(&mut self.auto_update, "Auto update?");
      ui.end_row();
      return update;
    }).body_returned.map(|i| i.inner).unwrap_or(false) || self.auto_update
  }

  pub fn draw_lod_render_data_gui(
    &mut self,
    ui: &mut Ui,
    lod_render_data: &LodRenderData,
  ) {
    ui.collapsing_with_grid("LOD render data", "Grid", |ui| {
      ui.label("# vertices");
      ui.monospace(format!("{}", lod_render_data.vertex_buffer.len));
      ui.end_row();
      ui.label("Vertex buffer size");
      ui.monospace(format!("{}", lod_render_data.vertex_buffer.size));
      ui.end_row();
      ui.label("# indices");
      ui.monospace(format!("{}", lod_render_data.index_buffer.len));
      ui.end_row();
      ui.label("Index buffer size");
      ui.monospace(format!("{}", lod_render_data.index_buffer.size));
      ui.end_row();
      ui.label("# draw commands");
      ui.monospace(format!("{}", lod_render_data.draws.len()));
      ui.end_row();
    });
  }

  pub fn draw_stars_renderer_settings(
    &mut self,
    ui: &mut Ui,
  ) {
    ui.collapsing_with_grid("Stars renderer", "Grid", |ui| {
      ui.label("Stars threshold");
      ui.drag_unlabelled_range(&mut self.stars_renderer_settings.stars_threshold, 0.1, 0.0..=1000.0);
      ui.end_row();
      ui.label("Stars exposure");
      ui.drag_unlabelled_range(&mut self.stars_renderer_settings.stars_exposure, 0.1, 0.0..=1000.0);
      ui.end_row();
      ui.label("Stars noise frequency");
      ui.drag_unlabelled_range(&mut self.stars_renderer_settings.stars_noise_frequency, 0.5, 0.0..=2000.0);
      ui.end_row();
      ui.label("Temperature noise frequency");
      ui.drag_unlabelled_range(&mut self.stars_renderer_settings.temperature_noise_frequency, 0.5, 0.0..=2000.0);
      ui.end_row();
      ui.label("Temperature minimum");
      ui.drag_unlabelled_range(&mut self.stars_renderer_settings.temperature_minimum, 1.0, 0.0..=100000.0);
      ui.end_row();
      ui.label("Temperature maximum");
      ui.drag_unlabelled_range(&mut self.stars_renderer_settings.temperature_maximum, 1.0, 0.0..=100000.0);
      ui.end_row();
      ui.label("Temperature power");
      ui.drag_unlabelled_range(&mut self.stars_renderer_settings.temperature_power, 0.01, 0.0..=20.0);
      ui.end_row();
    });
  }
}
