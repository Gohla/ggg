///! Render a couple of quads. Mostly made by following: https://sotrh.github.io/learn-wgpu/beginner/tutorial8-depth/.

use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use egui::Ui;
use serde::{Deserialize, Serialize};
use ultraviolet::{Isometry3, Mat4, Rotor3, Vec2, Vec3};
use wgpu::{BufferAddress, CommandBuffer, IndexFormat, RenderPipeline, ShaderStages, VertexAttribute, VertexBufferLayout, VertexStepMode};

use app::{AppRunner, RenderInput};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::{Gfx, include_spirv_shader_for_bin};
use gfx::bind_group::{CombinedBindGroup, CombinedBindGroupBuilder};
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{Camera, CameraData, CameraSettings};
use gfx::camera::controller::CameraControllerInput;
use gfx::camera::inspector::CameraInspector;
use gfx::sampler::SamplerBuilder;
use gfx::texture::TextureBuilder;
use os::Os;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pos: Vec2,
  tex: Vec2,
}
impl Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];
    VertexBufferLayout {
      array_stride: size_of::<Vertex>() as BufferAddress,
      step_mode: VertexStepMode::Vertex,
      attributes: ATTRIBUTES,
    }
  }
}

const VERTICES: &[Vertex] = &[
  Vertex { pos: Vec2::new(-0.5, -0.5), tex: Vec2::new(0.0, 1.0) },
  Vertex { pos: Vec2::new(0.5, -0.5), tex: Vec2::new(1.0, 1.0) },
  Vertex { pos: Vec2::new(-0.5, 0.5), tex: Vec2::new(0.0, 0.0) },
  Vertex { pos: Vec2::new(0.5, 0.5), tex: Vec2::new(1.0, 0.0) },
];

const INDICES: &[u16] = &[
  0, 1, 2,
  1, 3, 2
];

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  view_projection: Mat4,
}
impl Uniform {
  pub fn from_camera(camera: &Camera) -> Self {
    Self {
      view_projection: *camera.view_projection_matrix(),
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Instance {
  model: Mat4,
}
impl Instance {
  fn from_isometry(isometry: Isometry3) -> Self { Self { model: isometry.into_homogeneous_matrix() } }

  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4];
    VertexBufferLayout {
      array_stride: size_of::<Instance>() as BufferAddress,
      step_mode: VertexStepMode::Instance,
      attributes: ATTRIBUTES,
    }
  }
}

const NUM_INSTANCES_PER_ROW: u32 = 10;
const NUM_INSTANCES: u32 = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;

pub struct Quads {
  data: Data,

  camera: Camera,

  diffuse_bind_group: CombinedBindGroup,

  uniform_buffer: GfxBuffer,
  uniform_bind_group: CombinedBindGroup,

  render_pipeline: RenderPipeline,

  vertex_buffer: GfxBuffer,
  index_buffer: GfxBuffer,
  instance_buffer: GfxBuffer,
}

#[derive(Default, Copy, Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Data {
  camera_data: CameraData,
  camera_settings: CameraSettings,
  camera_inspector: CameraInspector,
}
impl Data {
  pub fn set_camera_inspector_defaults(&mut self) {
    let default = Self::default();
    self.camera_inspector.default_data = default.camera_data;
    self.camera_inspector.default_settings = default.camera_settings;
  }
}

pub struct Input {
  camera: CameraControllerInput,
}

impl app::Application for Quads {
  type Data = Data;
  fn new(_os: &Os, gfx: &Gfx, screen_size: ScreenSize, mut data: Self::Data) -> Self {
    data.set_camera_inspector_defaults();

    let camera = Camera::new(&mut data.camera_data, &data.camera_settings, screen_size.physical);

    let diffuse_bind_group = {
      let image = image::load_from_memory(include_bytes!("../../../../assets/alias3/construction_materials/cobble_stone_1.png")).unwrap().into_rgba8();
      let texture = TextureBuilder::new_from_2d_rgba_image(&image)
        .with_texture_label("Cobblestone diffuse texture")
        .with_texture_view_label("Cobblestone diffuse texture view")
        .build(&gfx.device);
      texture.write_2d_rgba_image(&gfx.queue, image);
      let sampler = SamplerBuilder::default()
        .label("Cobblestone diffuse sampler")
        .build(&gfx.device);
      let texture_binding = texture.binding(0, ShaderStages::FRAGMENT);
      let sampler_binding = sampler.binding(1, ShaderStages::FRAGMENT);
      CombinedBindGroupBuilder::default()
        .layout_entries(&[texture_binding.layout, sampler_binding.layout])
        .entries(&[texture_binding.entry, sampler_binding.entry])
        .layout_label("Cobblestone diffuse bind group layout")
        .label("Cobblestone diffuse bind group")
        .build(&gfx.device)
    };

    let uniform_buffer = BufferBuilder::default()
      .uniform_usage()
      .build_with_data(&gfx.device, &[Uniform::from_camera(&camera)]);
    let uniform_binding = uniform_buffer.binding(0, ShaderStages::VERTEX);
    let uniform_bind_group = CombinedBindGroupBuilder::new()
      .layout_entries(&[uniform_binding.layout])
      .entries(&[uniform_binding.entry])
      .layout_label("Quads uniform bind group layout")
      .label("Quads uniform bind group")
      .build(&gfx.device);
    let uniform_buffer = uniform_buffer;

    let vertex_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("vert"));
    let fragment_shader_module = gfx.device.create_shader_module(include_spirv_shader_for_bin!("frag"));

    let (_, render_pipeline) = gfx.render_pipeline_builder()
      .layout_label("Quads pipeline layout")
      .bind_group_layouts(&[&diffuse_bind_group.layout, &uniform_bind_group.layout])
      .label("Quads render pipeline")
      .vertex_module(&vertex_shader_module)
      .vertex_buffer_layouts(&[Vertex::buffer_layout(), Instance::buffer_layout()])
      .fragment_module(&fragment_shader_module)
      .build(&gfx.device);
    let vertex_buffer = BufferBuilder::default()
      .static_vertex_usage()
      .label("Quad static vertex buffer")
      .build_with_data(&gfx.device, VERTICES);
    let index_buffer = BufferBuilder::default()
      .static_index_usage()
      .label("Quad static index buffer")
      .build_with_data(&gfx.device, INDICES);
    let instances: Vec<Instance> = (0..NUM_INSTANCES_PER_ROW).flat_map(|y| {
      let y = y as f32 - (NUM_INSTANCES_PER_ROW as f32 / 2.0);
      (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        let x = x as f32 - (NUM_INSTANCES_PER_ROW as f32 / 2.0);
        let translation = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);
        let rotation = Rotor3::from_euler_angles(0.0, x, y);
        Instance::from_isometry(Isometry3::new(translation, rotation))
      })
    }).collect();
    let instance_buffer = BufferBuilder::default()
      .static_vertex_usage()
      .label("Quads instance buffer")
      .build_with_data(&gfx.device, &instances);

    Self {
      data,

      camera,

      diffuse_bind_group,

      uniform_buffer,
      uniform_bind_group,

      render_pipeline,

      vertex_buffer,
      index_buffer,
      instance_buffer,
    }
  }
  fn into_data(self) -> Self::Data {
    self.data
  }

  fn viewport_resize(&mut self, _os: &Os, _gfx: &Gfx, viewport: ScreenSize) {
    self.camera.set_viewport(viewport.physical);
  }

  type Input = Input;
  fn process_input(&mut self, input: RawInput) -> Self::Input {
    let camera = CameraControllerInput::from(&input);
    Input { camera }
  }

  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    self.data.camera_inspector.add_to_menu(ui);
  }

  fn render(&mut self, RenderInput { gfx, frame, input, gfx_frame, gui, .. }: RenderInput<Self>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.data.camera_inspector.show_window_single(&gui, &mut self.camera, &mut self.data.camera_data, &mut self.data.camera_settings);
    self.camera.update(&mut self.data.camera_data, &self.data.camera_settings, &input.camera, frame.duration);
    self.uniform_buffer.write_all_data(&gfx.queue, &[Uniform::from_camera(&self.camera)]);

    gui.window("Quads").show(&gui, |ui| {
      ui.label("Hello, world!");
    });

    let mut pass = gfx_frame.render_pass_builder()
      .label("Quads render pass")
      .begin();
    pass.push_debug_group("Draw quads");
    pass.set_pipeline(&self.render_pipeline);
    pass.set_bind_group(0, &self.diffuse_bind_group.entry, &[]);
    pass.set_bind_group(1, &self.uniform_bind_group.entry, &[]);
    pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
    pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    pass.draw_indexed(0..INDICES.len() as u32, 0, 0..NUM_INSTANCES);
    pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  AppRunner::from_name("Quads")
    .run::<Quads>()
    .unwrap();
}
