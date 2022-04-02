use egui::{ComboBox, Ui};
use egui::color_picker::Alpha;
use ultraviolet::Isometry3;

use gui_widget::UiWidgetsExt;
use voxel::chunk::GenericChunkSize;
use voxel::lod::chunk::LodChunkManager;
use voxel::lod::mesh::{LodMesh, LodMeshManagerSettings};
use voxel::lod::octmap::{LodOctmap, LodOctmapSettings};
use voxel::marching_cubes::MarchingCubes;
use voxel::transvoxel::Transvoxel;
use voxel::uniform::LightSettings;
use voxel::volume::{Noise, NoiseSettings, Plus, Sphere, SphereSettings};

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
pub enum MeshingAlgorithmType {
  MarchingCubes,
}

impl Default for MeshingAlgorithmType {
  fn default() -> Self { Self::MarchingCubes }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Settings {
  pub light: LightSettings,

  pub volume_type: VolumeType,
  pub sphere_settings: SphereSettings,
  pub noise_settings: NoiseSettings,

  pub meshing_algorithm_type: MeshingAlgorithmType,

  pub lod_octmap_settings: LodOctmapSettings,
  pub lod_octmap_transform: Isometry3,

  pub lod_mesh_manager_settings: LodMeshManagerSettings,
  pub auto_update: bool,
}

impl Settings {
  pub fn create_lod_chunk_manager(&self) -> Box<dyn LodChunkManager> {
    match self.volume_type {
      VolumeType::Sphere => Box::new(LodOctmap::new(self.lod_octmap_settings, self.lod_octmap_transform, Sphere::new(self.sphere_settings), MarchingCubes::<GenericChunkSize<16>>::new(), Transvoxel::<GenericChunkSize<16>>::new())),
      VolumeType::Noise => Box::new(LodOctmap::new(self.lod_octmap_settings, self.lod_octmap_transform, Noise::new(self.noise_settings), MarchingCubes::<GenericChunkSize<16>>::new(), Transvoxel::<GenericChunkSize<16>>::new())),
      VolumeType::SpherePlusNoise => Box::new(LodOctmap::new(self.lod_octmap_settings, self.lod_octmap_transform, Plus::new(Sphere::new(self.sphere_settings), Noise::new(self.noise_settings)), MarchingCubes::<GenericChunkSize<16>>::new(), Transvoxel::<GenericChunkSize<16>>::new())),
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
  pub fn draw_meshing_algorithm_gui(&mut self, ui: &mut Ui) -> bool {
    ui.collapsing_open_with_grid("Meshing Algorithm", "Grid", |ui| {
      ui.label("Type");
      ComboBox::from_id_source("Type")
        .selected_text(format!("{:?}", self.meshing_algorithm_type))
        .show_ui(ui, |ui| {
          ui.selectable_value(&mut self.meshing_algorithm_type, MeshingAlgorithmType::MarchingCubes, "Marching Cubes");
        });
      ui.end_row();
      match self.meshing_algorithm_type {
        MeshingAlgorithmType::MarchingCubes => {}
      }
      return ui.button("Update").clicked();
    }).body_returned.map(|i| i.inner).unwrap_or(false)
  }

  pub fn draw_lod_chunk_manager_gui(&mut self, ui: &mut Ui, lod_chunk_manager: &mut dyn LodChunkManager) {
    ui.collapsing_open_with_grid("LOD chunk manager", "Grid", |ui| {
      ui.label("LOD factor");
      ui.drag_unlabelled_range(lod_chunk_manager.get_lod_factor_mut(), 0.1, 0.0..=4.0);
      ui.end_row();
      ui.label("Max LOD level");
      ui.monospace(format!("{}", lod_chunk_manager.get_max_lod_level()));
      ui.end_row();
    });
  }

  /// Returns true if update button was pressed or auto update is set to true.
  pub fn draw_lod_mesh_manager_gui(
    &mut self,
    ui: &mut Ui,
  ) -> bool {
    ui.collapsing_open_with_grid("LOD mesh manager", "Grid", |ui| {
      ui.label("Render regular chunks?");
      ui.checkbox(&mut self.lod_mesh_manager_settings.render_regular_chunks, "");
      ui.end_row();
      ui.label("Render transition chunks?");
      ui.grid("Transition cell rendering", |ui| {
        ui.checkbox(&mut self.lod_mesh_manager_settings.render_transition_lo_x_chunks, "Lo X");
        ui.checkbox(&mut self.lod_mesh_manager_settings.render_transition_hi_x_chunks, "Hi X");
        ui.end_row();
        ui.checkbox(&mut self.lod_mesh_manager_settings.render_transition_lo_y_chunks, "Lo Y");
        ui.checkbox(&mut self.lod_mesh_manager_settings.render_transition_hi_y_chunks, "Hi Y");
        ui.end_row();
        ui.checkbox(&mut self.lod_mesh_manager_settings.render_transition_lo_z_chunks, "Lo Z");
        ui.checkbox(&mut self.lod_mesh_manager_settings.render_transition_hi_z_chunks, "Hi Z");
        ui.end_row();
      });
      ui.end_row();
      ui.label("Debug render octree nodes?");
      ui.grid("Debug rendering", |ui| {
        ui.checkbox(&mut self.lod_mesh_manager_settings.debug_render_octree_nodes, "");
        ui.edit_color_vec4(&mut self.lod_mesh_manager_settings.debug_render_octree_node_color, Alpha::OnlyBlend);
        ui.edit_color_vec4(&mut self.lod_mesh_manager_settings.debug_render_octree_node_empty_color, Alpha::OnlyBlend);
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

  pub fn draw_lod_mesh_gui(
    &mut self,
    ui: &mut Ui,
    lod_mesh: &LodMesh,
  ) {
    ui.collapsing_open_with_grid("LOD mesh", "Grid", |ui| {
      ui.label("# vertices");
      ui.monospace(format!("{}", lod_mesh.vertex_buffer.len));
      ui.end_row();
      ui.label("Vertex buffer size");
      ui.monospace(format!("{}", lod_mesh.vertex_buffer.size));
      ui.end_row();
      ui.label("# indices");
      ui.monospace(format!("{}", lod_mesh.index_buffer.len));
      ui.end_row();
      ui.label("Index buffer size");
      ui.monospace(format!("{}", lod_mesh.index_buffer.size));
      ui.end_row();
      ui.label("# draw commands");
      ui.monospace(format!("{}", lod_mesh.draws.len()));
      ui.end_row();
    });
  }
}
