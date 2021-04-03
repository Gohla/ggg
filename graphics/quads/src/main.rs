use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use ultraviolet::Vec2;
use wgpu::{BindGroup, Buffer, BufferAddress, CommandBuffer, include_spirv, IndexFormat, InputStepMode, PipelineLayout, RenderPipeline, ShaderModule, VertexAttribute, VertexBufferLayout};

use app::{Frame, Gfx, Os, Tick};
use gfx::buffer::DeviceBufferEx;
use gfx::command::DeviceCommandEncoderEx;
use gfx::render_pass::RenderPassBuilder;
use gfx::render_pipeline::RenderPipelineBuilder;
use os::input_sys::RawInput;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
  pos: Vec2,
  tex: Vec2,
}

impl Vertex {
  fn buffer_layout() -> VertexBufferLayout<'static> {
    const ATTRIBUTES: &[VertexAttribute] = &wgpu::vertex_attr_array![0 => Float2, 1 => Float2];
    VertexBufferLayout {
      array_stride: size_of::<Vertex>() as BufferAddress,
      step_mode: InputStepMode::Vertex,
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

pub struct Triangle {
  _vertex_shader_module: ShaderModule,
  _fragment_shader_module: ShaderModule,
  diffuse_bind_group: BindGroup,
  _pipeline_layout: PipelineLayout,
  render_pipeline: RenderPipeline,
  vertex_buffer: Buffer,
  index_buffer: Buffer,
}

impl app::App for Triangle {
  fn new(_os: &Os, gfx: &Gfx) -> Self {
    let vertex_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/cube.vert.spv"));
    let fragment_shader_module = gfx.device.create_shader_module(&include_spirv!("../../../target/shader/cube.frag.spv"));

    let diffuse_image = image::load_from_memory(include_bytes!("../../../assets/cobble_stone.bmp")).unwrap();
    let diffuse_rgba = diffuse_image.into_rgba8();
    let dimensions = diffuse_rgba.dimensions();
    let texture_size = wgpu::Extent3d {
      width: dimensions.0,
      height: dimensions.1,
      depth: 1,
    };
    let diffuse_texture = gfx.device.create_texture(
      &wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        label: None,
      }
    );
    gfx.queue.write_texture(
      wgpu::TextureCopyView {
        texture: &diffuse_texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
      },
      diffuse_rgba.as_raw(),
      wgpu::TextureDataLayout {
        offset: 0,
        bytes_per_row: 4 * dimensions.0,
        rows_per_image: dimensions.1,
      },
      texture_size,
    );
    let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let diffuse_sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Nearest,
      mipmap_filter: wgpu::FilterMode::Nearest,
      ..Default::default()
    });
    let texture_bind_group_layout = gfx.device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::Texture {
              multisampled: false,
              view_dimension: wgpu::TextureViewDimension::D2,
              sample_type: wgpu::TextureSampleType::Float { filterable: false },
            },
            count: None,
          },
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::Sampler {
              comparison: false,
              filtering: true,
            },
            count: None,
          },
        ],
        label: None,
      }
    );
    let diffuse_bind_group = gfx.device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
          }
        ],
        label: Some("diffuse_bind_group"),
      }
    );

    let (pipeline_layout, render_pipeline) = RenderPipelineBuilder::new(&vertex_shader_module)
      .with_bind_group_layouts(&[&texture_bind_group_layout])
      .with_default_fragment_state(&fragment_shader_module, &gfx.swap_chain)
      .with_vertex_buffer_layouts(&[Vertex::buffer_layout()])
      .build(&gfx.device);
    let vertex_buffer = gfx.device.create_vertex_buffer(VERTICES);
    let index_buffer = gfx.device.create_index_buffer(INDICES);

    Self {
      _vertex_shader_module: vertex_shader_module,
      _fragment_shader_module: fragment_shader_module,
      diffuse_bind_group,
      _pipeline_layout: pipeline_layout,
      render_pipeline,
      vertex_buffer,
      index_buffer,
    }
  }

  fn process_input(&mut self, _raw_input: RawInput) {}

  fn simulate(&mut self, _tick: Tick) {}

  fn render<'a>(&mut self, _os: &Os, gfx: &Gfx, frame: Frame<'a>) -> Box<dyn Iterator<Item=CommandBuffer>> {
    let mut encoder = gfx.device.create_default_command_encoder();
    {
      let mut render_pass = RenderPassBuilder::new()
        .begin_render_pass_for_swap_chain(&mut encoder, &frame.output_texture);
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
      render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }
    Box::new(std::iter::once(encoder.finish()))
  }
}

fn main() { app::run_with_defaults::<Triangle>("Cubes").unwrap(); }
