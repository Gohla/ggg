use std::num::NonZeroU32;

use wgpu::{
  BindGroupLayout, ColorTargetState, CompareFunction, DepthBiasState, DepthStencilState, Device, Face, FragmentState,
  FrontFace, MultisampleState, PipelineCompilationOptions, PipelineLayout, PolygonMode,
  PrimitiveState, PrimitiveTopology, PushConstantRange, RenderPipeline, RenderPipelineDescriptor, ShaderModule,
  StencilState, TextureFormat, VertexBufferLayout, VertexState,
};

use crate::fragment_state::FragmentStateBuilder;
use crate::pipeline_layout::PipelineLayoutBuilder;
use crate::surface::GfxSurface;
use crate::vertex_state::VertexStateBuilder;

pub struct RenderPipelineBuilder<'a> {
  layout: PipelineLayoutBuilder<'a>,
  // Note: not storing `RenderPipelineDescriptor` here due to it not being well-suited for a builder.
  label: Option<&'a str>,
  vertex: VertexStateBuilder<'a>,
  primitive: PrimitiveState,
  depth_stencil: Option<DepthStencilState>,
  multisample: MultisampleState,
  fragment: FragmentStateBuilder<'a>,
  multiview: Option<NonZeroU32>,
}
impl<'a> Default for RenderPipelineBuilder<'a> {
  #[inline]
  fn default() -> Self {
    Self {
      layout: PipelineLayoutBuilder::default(),
      label: None,
      vertex: VertexStateBuilder::default(),
      primitive: PrimitiveState {
        front_face: FrontFace::Cw, // TODO: the default is counter clockwise!?
        ..PrimitiveState::default()
      },
      depth_stencil: None,
      multisample: MultisampleState::default(),
      fragment: FragmentStateBuilder::default(),
      multiview: None,
    }
  }
}


impl<'a> RenderPipelineBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[inline]
  pub fn with_layout_label(mut self, label: &'a str) -> Self {
    self.layout = self.layout.label(label);
    self
  }
  #[inline]
  pub fn with_bind_group_layouts(mut self, bind_group_layouts: &'a [&'a BindGroupLayout]) -> Self {
    self.layout = self.layout.bind_group_layouts(bind_group_layouts);
    self
  }
  #[inline]
  pub fn with_push_constant_ranges(mut self, push_constant_ranges: &'a [PushConstantRange]) -> Self {
    self.layout = self.layout.push_constant_ranges(push_constant_ranges);
    self
  }

  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  #[inline]
  pub fn with_vertex(mut self, vertex: VertexState<'a>) -> Self {
    self.vertex = vertex.into();
    self
  }
  #[inline]
  pub fn with_vertex_builder(mut self, vertex: VertexStateBuilder<'a>) -> Self {
    self.vertex = vertex;
    self
  }
  #[inline]
  pub fn with_vertex_module(mut self, module: &'a ShaderModule) -> Self {
    self.vertex = self.vertex.module(module);
    self
  }
  #[inline]
  pub fn with_vertex_entry_point(mut self, entry_point: &'a str) -> Self {
    self.vertex = self.vertex.entry_point(entry_point);
    self
  }
  #[inline]
  pub fn with_vertex_compiler_options(mut self, compilation_options: PipelineCompilationOptions<'a>) -> Self {
    self.vertex = self.vertex.compilation_options(compilation_options);
    self
  }
  #[inline]
  pub fn with_vertex_buffer_layouts(mut self, buffer_layouts: &'a [VertexBufferLayout<'a>]) -> Self {
    self.vertex = self.vertex.buffer_layouts(buffer_layouts);
    self
  }

  #[inline]
  pub fn with_primitive(mut self, primitive: PrimitiveState) -> Self {
    self.primitive = primitive;
    self
  }
  #[inline]
  pub fn with_primitive_topology(mut self, primitive_topology: PrimitiveTopology) -> Self {
    self.primitive.topology = primitive_topology;
    self
  }
  #[inline]
  pub fn with_front_face(mut self, front_face: FrontFace) -> Self {
    self.primitive.front_face = front_face;
    self
  }
  #[inline]
  pub fn with_cull_mode(mut self, cull_mode: Option<Face>) -> Self {
    self.primitive.cull_mode = cull_mode;
    self
  }
  #[inline]
  pub fn with_polygon_mode(mut self, polygon_mode: PolygonMode) -> Self {
    self.primitive.polygon_mode = polygon_mode;
    self
  }

  #[inline]
  pub fn with_depth_stencil(mut self, depth_stencil: Option<DepthStencilState>) -> Self {
    self.depth_stencil = depth_stencil;
    self
  }
  #[inline]
  pub fn without_depth_stencil(self) -> Self {
    self.with_depth_stencil(None)
  }
  #[inline]
  pub fn with_depth_from_format(self, format: Option<TextureFormat>) -> Self {
    self.with_depth_stencil(format.map(|format| DepthStencilState {
      format,
      depth_write_enabled: true,
      depth_compare: CompareFunction::Greater, // Using "reversed Z", so depth compare using greater instead of less.
      stencil: StencilState::default(),
      bias: DepthBiasState::default(),
    }))
  }

  #[inline]
  pub fn with_multisample(mut self, multisample: MultisampleState) -> Self {
    self.multisample = multisample;
    self
  }
  #[inline]
  pub fn with_multisample_count(mut self, count: u32) -> Self {
    self.multisample.count = count;
    self
  }

  #[inline]
  pub fn with_fragment(mut self, fragment: FragmentState<'a>) -> Self {
    self.fragment = fragment.into();
    self
  }
  #[inline]
  pub fn with_fragment_builder(mut self, fragment: FragmentStateBuilder<'a>) -> Self {
    self.fragment = fragment;
    self
  }
  #[inline]
  pub fn with_fragment_module(mut self, module: &'a ShaderModule) -> Self {
    self.fragment = self.fragment.with_module(module);
    self
  }
  #[inline]
  pub fn with_fragment_entry_point(mut self, entry_point: &'a str) -> Self {
    self.fragment = self.fragment.with_entry_point(entry_point);
    self
  }
  #[inline]
  pub fn with_fragment_compiler_options(mut self, compilation_options: PipelineCompilationOptions<'a>) -> Self {
    self.fragment = self.fragment.with_compilation_options(compilation_options);
    self
  }
  #[inline]
  pub fn with_fragment_targets(mut self, targets: &'a [Option<ColorTargetState>]) -> Self {
    self.fragment = self.fragment.with_targets(targets);
    self
  }
  #[inline]
  pub fn with_surface_fragment_target(self, surface: &'a GfxSurface) -> Self {
    self.with_fragment_targets(&surface.non_blend_target)
  }
  #[inline]
  pub fn with_surface_replace_fragment_target(self, surface: &'a GfxSurface) -> Self {
    self.with_fragment_targets(&surface.replace_blend_target)
  }
  #[inline]
  pub fn with_surface_alpha_blend_fragment_target(self, surface: &'a GfxSurface) -> Self {
    self.with_fragment_targets(&surface.alpha_blend_target)
  }
  #[inline]
  pub fn with_surface_premultiplied_alpha_blend_fragment_target(self, surface: &'a GfxSurface) -> Self {
    self.with_fragment_targets(&surface.premultiplied_alpha_blend_target)
  }

  #[inline]
  pub fn with_multiview(mut self, multiview: Option<NonZeroU32>) -> Self {
    self.multiview = multiview;
    self
  }

  #[inline]
  pub fn build(self, device: &Device) -> (PipelineLayout, RenderPipeline) {
    let layout = self.layout.build(device);
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: self.label,
      layout: Some(&layout),
      vertex: self.vertex.build()
        .unwrap_or_else(|| panic!("Cannot build `RenderPipeline`: vertex shader module was not set")),
      primitive: self.primitive,
      depth_stencil: self.depth_stencil,
      multisample: self.multisample,
      fragment: self.fragment.into(),
      multiview: self.multiview,
    });
    (layout, pipeline)
  }
}
