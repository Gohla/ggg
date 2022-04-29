use bytemuck::{Pod, Zeroable};
use ultraviolet::{Mat4, Vec4};
use wgpu::{BindGroup, RenderPipeline, ShaderStages};

use common::screen::ScreenSize;
use gfx::{Frame, Gfx, include_shader_for_bin};
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::Camera;
use gfx::full_screen_triangle::FullScreenTriangle;
use gfx::render_pass::RenderPassBuilder;

pub struct StarsRenderer {
  uniform: Uniform,
  uniform_buffer: GfxBuffer,
  uniform_bind_group: BindGroup,
  full_screen_triangle: FullScreenTriangle,
  render_pipeline: RenderPipeline,
}

#[derive(Copy, Clone, Debug)]
pub struct StarsRendererSettings {
  pub stars_threshold: f32,
  pub stars_exposure: f32,
  pub stars_distance: f32,
}

impl Default for StarsRendererSettings {
  fn default() -> Self {
    Self {
      stars_threshold: 10.0,
      stars_exposure: 50.0,
      stars_distance: 500.0,
    }
  }
}

impl StarsRenderer {
  pub fn new(gfx: &Gfx, camera: &Camera) -> Self {
    let mut uniform = Uniform::default();
    uniform.update_screen_size(gfx.surface.get_size());
    uniform.update_view_projection(camera);
    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Stars uniform buffer")
      .build_with_data(&gfx.device, &[uniform]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStages::FRAGMENT);

    let (uniform_bind_group_layout, uniform_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry])
      .with_layout_label("Stars uniform bind group layout")
      .with_label("Stars uniform bind group")
      .build(&gfx.device);

    let full_screen_triangle = FullScreenTriangle::new(&gfx.device);
    let fragment_shader_module = gfx.device.create_shader_module(&include_shader_for_bin!("stars/frag"));
    let (_, render_pipeline) = full_screen_triangle.create_render_pipeline_builder()
      .with_bind_group_layouts(&[&uniform_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.surface)
      .with_layout_label("Stars pipeline layout")
      .with_label("Stars render pipeline")
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
    frame: &mut Frame,
    camera: &Camera,
    settings: &StarsRendererSettings,
  ) {
    self.uniform.update_view_projection(camera);
    self.uniform.update_settings(settings);
    self.uniform_buffer.write_whole_data(&gfx.queue, &[self.uniform]);

    let mut render_pass = RenderPassBuilder::new()
      .with_label("Stars render pass")
      .begin_render_pass_for_gfx_frame_with_clear(gfx, frame, false);
    render_pass.push_debug_group("Render stars");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    self.full_screen_triangle.draw(&mut render_pass);
    render_pass.pop_debug_group();
  }
}

#[repr(C)]
#[derive(Default, Copy, Clone, Pod, Zeroable, Debug)]
pub struct Uniform {
  screen_size: Vec4,
  view_projection: Mat4,
  stars_threshold: f32,
  stars_exposure: f32,
  stars_distance: f32,
  _dummy: f32,
}

impl Uniform {
  #[inline]
  pub fn update_screen_size(&mut self, screen_size: ScreenSize) {
    let screen_size = screen_size.physical;
    self.screen_size = Vec4::new(screen_size.width as f32, screen_size.height as f32, 0.0, 0.0);
  }

  #[inline]
  pub fn update_view_projection(&mut self, camera: &Camera) {
    self.view_projection = camera.get_view_inverse_matrix();
  }

  #[inline]
  pub fn update_settings(&mut self, settings: &StarsRendererSettings) {
    self.stars_threshold = settings.stars_threshold;
    self.stars_exposure = settings.stars_exposure;
    self.stars_distance = settings.stars_distance;
  }
}
