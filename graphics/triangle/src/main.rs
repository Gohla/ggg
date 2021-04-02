use wgpu::{Adapter, CommandBuffer, Device, Instance, PipelineLayout, Queue, RenderPipeline, ShaderModule, Surface};

use gfx::swap_chain::GfxSwapChain;
use os::input_sys::RawInput;
use os::window::OsWindow;
use util::timing::{Duration, FrameTime};

fn main() { app::run_with_defaults::<Triangle>("Triangle").unwrap(); }

pub struct Triangle {
  _vertex_shader_module: ShaderModule,
  _fragment_shader_module: ShaderModule,
  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,
}

impl app::App for Triangle {
  fn new(
    _window: &OsWindow,
    _instance: &Instance,
    _surface: &Surface,
    _adapter: &Adapter,
    device: &Device,
    _queue: &Queue,
    swap_chain: &GfxSwapChain,
  ) -> Self {
    let vertex_shader_module = device.create_shader_module(&wgpu::include_spirv!("../../../target/shader/triangle.vert.spv"));
    let fragment_shader_module = device.create_shader_module(&wgpu::include_spirv!("../../../target/shader/triangle.frag.spv"));
    let pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Triangle render pipeline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
      });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Triangle render pipeline"),
      layout: Some(&pipeline_layout),
      vertex: wgpu::VertexState {
        module: &vertex_shader_module,
        entry_point: "main",
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &fragment_shader_module,
        entry_point: "main",
        targets: &[wgpu::ColorTargetState {
          format: swap_chain.get_texture_format(), // TODO: need to recreate if swap chain changes!
          alpha_blend: wgpu::BlendState::REPLACE,
          color_blend: wgpu::BlendState::REPLACE,
          write_mask: wgpu::ColorWrite::ALL,
        }],
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: wgpu::CullMode::Back,
        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
        polygon_mode: wgpu::PolygonMode::Fill,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
    });
    Self {
      _vertex_shader_module: vertex_shader_module,
      _fragment_shader_module: fragment_shader_module,
      _pipeline_layout: pipeline_layout,
      render_pipeline,
    }
  }

  fn process_input(&mut self, _raw_input: RawInput) {}

  fn simulate(&mut self, _fixed_time_step: Duration) {}

  fn render(
    &mut self,
    _window: &OsWindow,
    _instance: &Instance,
    _surface: &Surface,
    _adapter: &Adapter,
    device: &Device,
    _queue: &Queue,
    _swap_chain: &GfxSwapChain,
    frame_output_texture: &wgpu::SwapChainTexture,
    _extrapolation: f64,
    _frame_time: FrameTime,
  ) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Triangle render encoder"),
    });
    {
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Triangle render pass"),
        color_attachments: &[
          wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &frame_output_texture.view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
              }),
              store: true,
            },
          }
        ],
        depth_stencil_attachment: None,
      });
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.draw(0..3, 0..1);
    }
    Box::new(std::iter::once(encoder.finish()))
  }
}
