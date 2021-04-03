use wgpu::{
  BlendState, Color, ColorTargetState, ColorWrite, CommandBuffer, CommandEncoderDescriptor, CullMode, FragmentState,
  FrontFace, include_spirv, LoadOp, MultisampleState, Operations, PipelineLayout, PipelineLayoutDescriptor, PolygonMode,
  PrimitiveState, PrimitiveTopology, RenderPassColorAttachmentDescriptor, RenderPassDescriptor, RenderPipeline,
  RenderPipelineDescriptor, ShaderModule, SwapChainTexture, VertexState
};

use app::{Gfx, Os};
use os::input_sys::RawInput;
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
    _os: &Os,
    gfx: &Gfx,
  ) -> Self {
    let vertex_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/triangle.vert.spv"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/triangle.frag.spv"));
    let pipeline_layout =
      gfx.device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Triangle render pipeline layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
      });
    let render_pipeline = gfx.device.create_render_pipeline(&RenderPipelineDescriptor {
      label: Some("Triangle render pipeline"),
      layout: Some(&pipeline_layout),
      vertex: VertexState {
        module: &vertex_shader_module,
        entry_point: "main",
        buffers: &[],
      },
      fragment: Some(FragmentState {
        module: &fragment_shader_module,
        entry_point: "main",
        targets: &[ColorTargetState {
          format: gfx.swap_chain.get_texture_format(), // TODO: need to recreate if swap chain changes!
          alpha_blend: BlendState::REPLACE,
          color_blend: BlendState::REPLACE,
          write_mask: ColorWrite::ALL,
        }],
      }),
      primitive: PrimitiveState {
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: FrontFace::Ccw,
        cull_mode: CullMode::Back,
        polygon_mode: PolygonMode::Fill,
      },
      depth_stencil: None,
      multisample: MultisampleState {
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
    _os: &Os,
    gfx: &Gfx,
    frame_output_texture: &SwapChainTexture,
    _extrapolation: f64,
    _frame_time: FrameTime,
  ) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let mut encoder = gfx.device.create_command_encoder(&CommandEncoderDescriptor {
      label: Some("Triangle render encoder"),
    });
    {
      let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("Triangle render pass"),
        color_attachments: &[
          RenderPassColorAttachmentDescriptor {
            attachment: &frame_output_texture.view,
            resolve_target: None,
            ops: Operations {
              load: LoadOp::Clear(Color {
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
