use wgpu::{BindGroupLayout, BlendState, ColorTargetState, CompareFunction, DepthStencilState, Device, Face, FragmentState, FrontFace, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, PushConstantRange, RenderPipeline, RenderPipelineDescriptor, ShaderModule, TextureFormat, VertexBufferLayout, VertexState};

use crate::surface::GfxSurface;

pub struct RenderPipelineBuilder<'a> {
  layout: PipelineLayoutDescriptor<'a>,
  label: Option<&'a str>,
  vertex: VertexState<'a>,
  primitive: PrimitiveState,
  depth_stencil: Option<DepthStencilState>,
  multisample: MultisampleState,
  fragment: Option<FragmentState<'a>>,
  default_fragment_targets: [Option<ColorTargetState>; 1],
  use_default_fragment_targets: bool,
}

impl<'a> RenderPipelineBuilder<'a> {
  pub fn new(
    vertex_shader_module: &'a ShaderModule,
  ) -> Self {
    Self {
      layout: PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
      },
      label: None,
      vertex: VertexState {
        module: vertex_shader_module,
        entry_point: "main",
        buffers: &[],
      },
      primitive: PrimitiveState {
        front_face: FrontFace::Cw,
        ..PrimitiveState::default()
      },
      depth_stencil: None,
      multisample: MultisampleState::default(),
      fragment: None,
      default_fragment_targets: [Some(ColorTargetState {
        format: TextureFormat::R8Unorm,
        blend: Default::default(),
        write_mask: Default::default(),
      })],
      use_default_fragment_targets: false,
    }
  }


  #[inline]
  pub fn with_layout_label(mut self, label: &'a str) -> Self {
    self.layout.label = Some(label);
    self
  }

  #[inline]
  pub fn with_bind_group_layouts(mut self, bind_group_layouts: &'a [&'a BindGroupLayout]) -> Self {
    self.layout.bind_group_layouts = bind_group_layouts;
    self
  }

  #[inline]
  pub fn with_push_constant_ranges(mut self, push_constant_ranges: &'a [PushConstantRange]) -> Self {
    self.layout.push_constant_ranges = push_constant_ranges;
    self
  }


  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }


  #[inline]
  pub fn with_vertex_state(mut self, vertex: VertexState<'a>) -> Self {
    self.vertex = vertex;
    self
  }

  #[inline]
  pub fn with_vertex_entry_point(mut self, entry_point: &'a str) -> Self {
    self.vertex.entry_point = entry_point;
    self
  }

  #[inline]
  pub fn with_vertex_buffer_layouts(mut self, buffer_layouts: &'a [VertexBufferLayout<'a>]) -> Self {
    self.vertex.buffers = buffer_layouts;
    self
  }


  #[inline]
  pub fn with_primitive_state(mut self, primitive: PrimitiveState) -> Self {
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
  pub fn with_depth_stencil(mut self, depth_stencil: DepthStencilState) -> Self {
    self.depth_stencil = Some(depth_stencil);
    self
  }

  #[inline]
  pub fn with_depth_texture(self, format: TextureFormat) -> Self {
    self.with_depth_stencil(DepthStencilState {
      format,
      depth_write_enabled: true,
      depth_compare: CompareFunction::Greater, // Using "reversed Z", so depth compare using greater instead of less.
      stencil: Default::default(),
      bias: Default::default(),
    })
  }


  #[inline]
  pub fn with_fragment_state(mut self, module: &'a ShaderModule, entry_point: &'a str, targets: &'a [Option<ColorTargetState>]) -> Self {
    self.fragment = Some(FragmentState {
      module,
      entry_point,
      targets,
    });
    self.use_default_fragment_targets = false;
    self
  }

  #[inline]
  pub fn with_default_fragment_state(mut self, module: &'a ShaderModule, surface: &GfxSurface) -> Self {
    self.fragment = Some(FragmentState {
      module,
      entry_point: "main",
      targets: &[],
    });
    if let Some(target) = &mut self.default_fragment_targets[0] {
      target.format = surface.get_texture_format();
    }
    self.use_default_fragment_targets = true;
    self
  }

  #[inline]
  pub fn with_default_alpha_blending_fragment_state(mut self, module: &'a ShaderModule, surface: &GfxSurface) -> Self {
    self.fragment = Some(FragmentState {
      module,
      entry_point: "main",
      targets: &[],
    });
    if let Some(target) = &mut self.default_fragment_targets[0] {
      target.format = surface.get_texture_format();
      target.blend = Some(BlendState::ALPHA_BLENDING);
    }
    self.use_default_fragment_targets = true;
    self
  }

  #[inline]
  pub fn with_default_premultiplied_alpha_blending_fragment_state(mut self, module: &'a ShaderModule, surface: &GfxSurface) -> Self {
    self.fragment = Some(FragmentState {
      module,
      entry_point: "main",
      targets: &[],
    });
    if let Some(target) = &mut self.default_fragment_targets[0] {
      target.format = surface.get_texture_format();
      target.blend = Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING);
    }
    self.use_default_fragment_targets = true;
    self
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


  pub fn build(self, device: &Device) -> (PipelineLayout, RenderPipeline) {
    let layout = device.create_pipeline_layout(&self.layout);
    let mut fragment = self.fragment;
    if self.use_default_fragment_targets {
      if let Some(ref mut fragment) = fragment {
        fragment.targets = &self.default_fragment_targets;
      }
    }
    let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
      label: self.label,
      layout: Some(&layout),
      vertex: self.vertex,
      primitive: self.primitive,
      depth_stencil: self.depth_stencil,
      multisample: self.multisample,
      fragment,
      multiview: None,
    });
    (layout, pipeline)
  }
}
