use bytemuck::{Pod, Zeroable};
use egui::Ui;
use rand::Rng;
use ultraviolet::{Mat4, Vec3, Vec4};
use wgpu::{BackendBit, BindGroup, CommandBuffer, CullMode, include_spirv, IndexFormat, PipelineLayout, PowerPreference, RenderPipeline, ShaderModule, ShaderStage};

use app::{Frame, Gfx, GuiFrame, Options, Os};
use common::input::RawInput;
use common::screen::ScreenSize;
use gfx::bind_group::CombinedBindGroupLayoutBuilder;
use gfx::buffer::{BufferBuilder, GfxBuffer};
use gfx::camera::{CameraInput, CameraSys};
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use gfx::texture::{GfxTexture, TextureBuilder};

const NUM_CUBE_INDICES: usize = 3 * 6 * 2;
const NUM_CUBE_VERTICES: usize = 8;
const CUBE_INDICES: [u32; NUM_CUBE_INDICES] = [
  0, 2, 1, 2, 3, 1,
  5, 4, 1, 1, 4, 0,
  0, 4, 6, 0, 6, 2,
  6, 5, 7, 6, 4, 5,
  2, 6, 3, 6, 7, 3,
  7, 1, 3, 7, 5, 1,
];

const MAX_INSTANCES: usize = 10_000_000;
const MAX_INDICES: usize = MAX_INSTANCES * NUM_CUBE_INDICES;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniform {
  view_projection: Mat4,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Instance {
  position: Vec4,
}

impl Instance {
  fn from_position(position: Vec3) -> Self { Self { position: position.into_homogeneous_point() } }
}

pub struct Cubes {
  camera_sys: CameraSys,

  uniform_buffer: GfxBuffer,
  _instance_buffer: GfxBuffer,
  static_bind_group: BindGroup,

  _vertex_shader_module: ShaderModule,
  _fragment_shader_module: ShaderModule,

  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,

  depth_texture: GfxTexture,

  index_buffer: GfxBuffer,
}

pub struct Input {
  camera: CameraInput,
}

impl app::Application for Cubes {
  fn new(os: &Os, gfx: &Gfx) -> Self {
    let viewport = os.window.get_inner_size().physical;
    let mut camera_sys = CameraSys::with_defaults_perspective(viewport);
    camera_sys.panning_speed = 100.0;

    let uniform_buffer = BufferBuilder::new()
      .with_uniform_usage()
      .with_label("Cubes uniform buffer")
      .build_with_data(&gfx.device, &[Uniform { view_projection: camera_sys.get_view_projection_matrix() }]);
    let (uniform_bind_group_layout_entry, uniform_bind_group_entry) = uniform_buffer.create_uniform_binding_entries(0, ShaderStage::VERTEX);

    let instance_buffer = {
      let mut rng = rand::thread_rng();
      let data: Vec<_> = (0..MAX_INSTANCES).map(|_| {
        Instance::from_position(Vec3::new(rng.gen_range(-5000.0..5000.0), rng.gen_range(-5000.0..5000.0), rng.gen_range(-5000.0..5000.0)))
      }).collect();
      BufferBuilder::new()
        .with_storage_usage()
        .with_label("Cubes instance storage buffer")
        .build_with_data(&gfx.device, &data)
    };
    let (instance_bind_group_layout_entry, instance_bind_group_entry) = instance_buffer.create_storage_binding_entries(1, ShaderStage::VERTEX, true);

    let (static_bind_group_layout, static_bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[uniform_bind_group_layout_entry, instance_bind_group_layout_entry])
      .with_entries(&[uniform_bind_group_entry, instance_bind_group_entry])
      .with_layout_label("Cubes static bind group layout")
      .with_label("Cubes static bind group")
      .build(&gfx.device);

    let vertex_shader_module = gfx.device.create_shader_module(&include_spirv!(concat!(env!("OUT_DIR"), "/shader/cube.vert.spv")));
    let fragment_shader_module = gfx.device.create_shader_module(&include_spirv!(concat!(env!("OUT_DIR"), "/shader/cube.frag.spv")));

    let depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);

    let (pipeline_layout, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&static_bind_group_layout])
      .with_cull_mode(CullMode::None)
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_depth_texture(depth_texture.format)
      .with_layout_label("Cubes pipeline layout")
      .with_label("Cubes render pipeline")
      .build(&gfx.device);

    let index_buffer = {
      let data: Vec<_> = (0..MAX_INDICES).map(|i| {
        let cube = i / NUM_CUBE_INDICES;
        let cube_local = i % NUM_CUBE_INDICES;
        CUBE_INDICES[cube_local] + cube as u32 * NUM_CUBE_VERTICES as u32
      }).collect();
      BufferBuilder::new()
        .with_static_index_usage()
        .with_label("Cubes static index buffer")
        .build_with_data(&gfx.device, &data)
    };

    Self {
      camera_sys,

      uniform_buffer,
      _instance_buffer: instance_buffer,
      static_bind_group,

      _vertex_shader_module: vertex_shader_module,
      _fragment_shader_module: fragment_shader_module,

      _pipeline_layout: pipeline_layout,
      render_pipeline,

      depth_texture,

      index_buffer,
    }
  }


  fn screen_resize(&mut self, _os: &Os, gfx: &Gfx, screen_size: ScreenSize) {
    let viewport = screen_size.physical;
    self.camera_sys.viewport = viewport;
    self.depth_texture = TextureBuilder::new_depth_32_float(viewport).build(&gfx.device);
  }


  type Input = Input;

  fn process_input(&mut self, input: RawInput) -> Input {
    let camera = CameraInput::from(&input);
    Input { camera }
  }


  fn add_to_debug_menu(&mut self, ui: &mut Ui) {
    ui.checkbox(&mut self.camera_sys.show_debug_gui, "Camera");
  }

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, frame: Frame<'a>, gui_frame: &GuiFrame, input: &Input) -> Box<dyn Iterator<Item=CommandBuffer>> {
    self.camera_sys.update(&input.camera, frame.time.delta, &gui_frame);
    self.uniform_buffer.write_whole_data(&gfx.queue, &[Uniform { view_projection: self.camera_sys.get_view_projection_matrix() }]);

    let mut render_pass = RenderPassBuilder::new()
      .with_depth_texture(&self.depth_texture.view)
      .with_label("Cubes render pass")
      .begin_render_pass_for_swap_chain_with_clear(frame.encoder, &frame.output_texture);
    render_pass.push_debug_group("Draw cubes");
    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.set_bind_group(0, &self.static_bind_group, &[]);
    render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    render_pass.draw_indexed(0..MAX_INDICES as u32, 0, 0..1);
    render_pass.pop_debug_group();
    Box::new(std::iter::empty())
  }
}

fn main() {
  app::run::<Cubes>(Options {
    name: "Cubes".to_string(),
    graphics_backends: BackendBit::all(),
    graphics_adapter_power_preference: PowerPreference::HighPerformance,
    ..Options::default()
  }).unwrap();
}