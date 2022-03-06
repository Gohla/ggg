use egui::{color_picker, ComboBox, DragValue, Rgba, Ui};
use egui::color_picker::Alpha;
use ultraviolet::{Rotor3, Vec3, Vec4};

use gui_widget::UiWidgetsExt;
use voxel_meshing::chunk::{GenericChunkSize};
use voxel_meshing::marching_cubes::MarchingCubes;
use voxel_meshing::octree::{Octree, OctreeSettings, VolumeMeshManager};
use voxel_meshing::transvoxel::Transvoxel;
use voxel_meshing::volume::{Noise, NoiseSettings, Plus, Sphere, SphereSettings};

use crate::{LightUniform, MeshGeneration};

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
  pub light_rotation_x_degree: f32,
  pub light_rotation_y_degree: f32,
  pub light_rotation_z_degree: f32,
  pub light_uniform: LightUniform,

  pub volume_type: VolumeType,
  pub sphere_settings: SphereSettings,
  pub noise_settings: NoiseSettings,

  pub meshing_algorithm_type: MeshingAlgorithmType,

  pub octree_settings: OctreeSettings,
  pub auto_update: bool,
  pub render_regular_chunks: bool,
  pub render_transition_lo_x_chunks: bool,
  pub render_transition_hi_x_chunks: bool,
  pub render_transition_lo_y_chunks: bool,
  pub render_transition_hi_y_chunks: bool,
  pub render_transition_lo_z_chunks: bool,
  pub render_transition_hi_z_chunks: bool,
  pub debug_render_octree_nodes: bool,
  pub debug_render_octree_node_color: Vec4,
  pub debug_render_octree_node_empty_color: Vec4,
}

impl Settings {
  pub fn create_volume_mesh_manager(&self) -> Box<dyn VolumeMeshManager> {
    match self.volume_type {
      VolumeType::Sphere => Box::new(Octree::new(self.octree_settings, Sphere::new(self.sphere_settings), MarchingCubes::<GenericChunkSize<16>>::new(), Transvoxel::<GenericChunkSize<16>>::new())),
      VolumeType::Noise => Box::new(Octree::new(self.octree_settings, Noise::new(self.noise_settings), MarchingCubes::<GenericChunkSize<16>>::new(), Transvoxel::<GenericChunkSize<16>>::new())),
      VolumeType::SpherePlusNoise => Box::new(Octree::new(self.octree_settings, Plus::new(Sphere::new(self.sphere_settings), Noise::new(self.noise_settings)), MarchingCubes::<GenericChunkSize<16>>::new(), Transvoxel::<GenericChunkSize<16>>::new())),
    }
  }

  pub fn render_gui(&mut self, ui: &mut Ui, mesh_generation: &mut MeshGeneration, recreate_volume_mesh_manager: &mut bool, update_volume_mesh_manager: &mut bool) {
    ui.collapsing_open_with_grid("Directional Light", "Grid", |mut ui| {
      ui.label("Color");
      let mut color = Rgba::from_rgba_premultiplied(self.light_uniform.color.x, self.light_uniform.color.y, self.light_uniform.color.z, 0.0).into();
      color_picker::color_edit_button_srgba(&mut ui, &mut color, Alpha::Opaque);
      let color: Rgba = color.into();
      self.light_uniform.color = Vec3::new(color.r(), color.g(), color.b());
      ui.end_row();
      ui.label("Ambient");
      ui.add(DragValue::new(&mut self.light_uniform.ambient).speed(0.001).clamp_range(0.0..=1.0));
      ui.end_row();
      ui.label("Direction");
      ui.drag("x: ", &mut self.light_rotation_x_degree, 0.5);
      ui.drag("y: ", &mut self.light_rotation_y_degree, 0.5);
      ui.drag("z: ", &mut self.light_rotation_z_degree, 0.5);
      self.light_uniform.direction = Rotor3::from_euler_angles((self.light_rotation_z_degree % 360.0).to_radians(), (self.light_rotation_x_degree % 360.0).to_radians(), (self.light_rotation_y_degree % 360.0).to_radians()) * Vec3::one();
      ui.end_row();
    });
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
        VolumeType::Sphere => self.gui_sphere_settings(ui),
        VolumeType::Noise => self.gui_noise_settings(ui),
        VolumeType::SpherePlusNoise => {
          self.gui_sphere_settings(ui);
          self.gui_noise_settings(ui);
        }
      }
      if ui.button("Update").clicked() {
        *recreate_volume_mesh_manager = true;
      }
    });
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
      if ui.button("Update").clicked() {
        *recreate_volume_mesh_manager = true;
      }
    });
    ui.collapsing_open_with_grid("Volume mesh manager", "Grid", |ui| {
      ui.label("LOD factor");
      ui.drag_unlabelled_range(mesh_generation.volume_mesh_manager.get_lod_factor_mut(), 0.1, 0.0..=4.0);
      ui.end_row();
      ui.label("Max LOD level");
      ui.monospace(format!("{}", mesh_generation.volume_mesh_manager.get_max_lod_level()));
      ui.end_row();
      ui.label("# vertices");
      ui.monospace(format!("{}", mesh_generation.vertices.len()));
      ui.end_row();
      ui.label("Vertex buffer size");
      ui.monospace(format!("{}", mesh_generation.vertex_buffer.size));
      ui.end_row();
      ui.label("# indices");
      ui.monospace(format!("{}", mesh_generation.indices.len()));
      ui.end_row();
      ui.label("Index buffer size");
      ui.monospace(format!("{}", mesh_generation.index_buffer.size));
      ui.end_row();
      ui.label("# draw commands");
      ui.monospace(format!("{}", mesh_generation.draws.len()));
      ui.end_row();
      ui.label("Render regular chunks?");
      ui.checkbox(&mut self.render_regular_chunks, "");
      ui.end_row();
      ui.label("Render transition chunks?");
      ui.grid("Transition cell rendering", |ui| {
        ui.checkbox(&mut self.render_transition_lo_x_chunks, "Lo X");
        ui.checkbox(&mut self.render_transition_hi_x_chunks, "Hi X");
        ui.end_row();
        ui.checkbox(&mut self.render_transition_lo_y_chunks, "Lo Y");
        ui.checkbox(&mut self.render_transition_hi_y_chunks, "Hi Y");
        ui.end_row();
        ui.checkbox(&mut self.render_transition_lo_z_chunks, "Lo Z");
        ui.checkbox(&mut self.render_transition_hi_z_chunks, "Hi Z");
        ui.end_row();
      });
      ui.end_row();
      ui.label("Debug render octree nodes?");
      ui.grid("Debug rendering", |ui| {
        ui.checkbox(&mut self.debug_render_octree_nodes, "");
        ui.edit_color_vec4(&mut self.debug_render_octree_node_color, Alpha::OnlyBlend);
        ui.edit_color_vec4(&mut self.debug_render_octree_node_empty_color, Alpha::OnlyBlend);
      });
      ui.end_row();
      if ui.button("Update").clicked() {
        *update_volume_mesh_manager = true;
      }
      ui.checkbox(&mut self.auto_update, "Auto update?");
      ui.end_row();
    });
  }

  fn gui_sphere_settings(&mut self, ui: &mut Ui) {
    ui.label("Radius");
    ui.drag_unlabelled(&mut self.sphere_settings.radius, 0.1);
    ui.end_row();
  }

  fn gui_noise_settings(&mut self, ui: &mut Ui) {
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
}
