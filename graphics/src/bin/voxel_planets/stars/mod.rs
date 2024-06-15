use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use ultraviolet::{Mat4, Vec4};
use wgpu::{RenderPipeline, ShaderStages};

use common::screen::ScreenSize;
use gfx::{Gfx, include_spirv_shader_for_bin, Render};
use gfx::bind_group::{CombinedBindGroup, CombinedBindGroupBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::full_screen_triangle::FullScreenTriangle;
use gfx::render_pass::RenderPassBuilder;

pub struct StarsRenderer {
  uniform: Uniform,
  uniform_buffer: GfxBuffer,
  uniform_bind_group: CombinedBindGroup,
  full_screen_triangle: FullScreenTriangle,
  render_pipeline: RenderPipeline,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct StarsRendererSettings {
  pub stars_threshold: f32,
  pub stars_exposure: f32,
  pub stars_noise_frequency: f32,
  pub temperature_noise_frequency: f32,
  pub temperature_minimum: f32,
  pub temperature_maximum: f32,
  pub temperature_power: f32,
}

impl Default for StarsRendererSettings {
  fn default() -> Self {
    Self {
      stars_threshold: 10.0,
      stars_exposure: 50.0,
      stars_noise_frequency: 500.0,
      temperature_noise_frequency: 100.0,
      temperature_minimum: 1500.0,
      temperature_maximum: 65000.0,
      temperature_power: 4.0,
    }
  }
}

impl StarsRenderer {
  pub fn new(gfx: &Gfx, view_inverse_matrix: Mat4) -> Self {
    let mut uniform = Uniform::default();
    uniform.update_screen_size(gfx.surface.get_size());
    uniform.view_inverse_matrix = view_inverse_matrix;
    let uniform_buffer = BufferBuilder::default()
      .label("Stars uniform buffer")
      .uniform_usage()
      .build_with_data(&gfx.device, &[uniform]);
    let uniform_binding = uniform_buffer.binding(0, ShaderStages::FRAGMENT);

    let uniform_bind_group = CombinedBindGroupBuilder::new()
      .layout_label("Stars uniform bind group layout")
      .layout_entries(&[uniform_binding.layout])
      .label("Stars uniform bind group")
      .entries(&[uniform_binding.entry])
      .build(&gfx.device);

    let full_screen_triangle = FullScreenTriangle::new(&gfx.device);
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("stars/frag"));
    let (_, render_pipeline) = full_screen_triangle.create_render_pipeline_builder(&gfx)
      .layout_label("Stars pipeline layout")
      .bind_group_layouts(&[&uniform_bind_group.layout])
      .label("Stars render pipeline")
      .fragment_module(&fragment_shader_module)
      .build(&gfx.device);

    Self {
      uniform,
      uniform_buffer,
      uniform_bind_group,
      full_screen_triangle,
      render_pipeline,
    }
  }

  pub fn screen_resize(&mut self, screen_size: ScreenSize) {
    self.uniform.update_screen_size(screen_size);
  }

  pub fn render(
    &mut self,
    gfx: &Gfx,
    frame: &mut Render,
    view_inverse_matrix: Mat4,
    settings: &StarsRendererSettings,
  ) {
    self.uniform.view_inverse_matrix = view_inverse_matrix;
    self.uniform.update_settings(settings);
    self.uniform_buffer.write_all_data(&gfx.queue, &[self.uniform]);

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Stars render pass")
      .begin_render_pass_for_gfx_frame_with_clear(gfx, frame, false);
    render_pass.push_debug_group("Render stars");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group.entry, &[]);
    self.full_screen_triangle.draw(&mut render_pass);
    render_pass.pop_debug_group();
  }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Pod, Zeroable, Debug)]
pub struct Uniform {
  screen_size: Vec4,
  view_inverse_matrix: Mat4,

  stars_threshold: f32,
  stars_exposure: f32,
  stars_noise_frequency: f32,
  temperature_noise_frequency: f32,

  temperature_minimum: f32,
  temperature_maximum: f32,
  temperature_power: f32,
  _dummy: f32,
}

impl Uniform {
  #[inline]
  pub fn update_screen_size(&mut self, screen_size: ScreenSize) {
    let screen_size = screen_size.physical;
    self.screen_size = Vec4::new(screen_size.width as f32, screen_size.height as f32, 0.0, 0.0);
  }

  #[inline]
  pub fn update_settings(&mut self, settings: &StarsRendererSettings) {
    self.stars_threshold = settings.stars_threshold;
    self.stars_exposure = settings.stars_exposure;
    self.stars_noise_frequency = settings.stars_noise_frequency;
    self.temperature_noise_frequency = settings.temperature_noise_frequency;
    self.temperature_minimum = settings.temperature_minimum;
    self.temperature_maximum = settings.temperature_maximum;
    self.temperature_power = settings.temperature_power;
  }
}
