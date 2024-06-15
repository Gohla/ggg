use wgpu::{PipelineCompilationOptions, ShaderModule, VertexBufferLayout, VertexState};

pub struct VertexStateBuilder<'a> {
  module: Option<&'a ShaderModule>,
  entry_point: &'a str,
  compilation_options: PipelineCompilationOptions<'a>,
  buffer_layouts: &'a [VertexBufferLayout<'a>],
}

impl<'a> Default for VertexStateBuilder<'a> {
  #[inline]
  fn default() -> Self {
    Self {
      module: None,
      entry_point: "main",
      compilation_options: PipelineCompilationOptions::default(),
      buffer_layouts: &[],
    }
  }
}

impl<'a> VertexStateBuilder<'a> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[inline]
  pub fn module(mut self, module: &'a ShaderModule) -> Self {
    self.module = Some(module);
    self
  }

  #[inline]
  pub fn entry_point(mut self, entry_point: &'a str) -> Self {
    self.entry_point = entry_point;
    self
  }

  #[inline]
  pub fn compilation_options(mut self, compilation_options: PipelineCompilationOptions<'a>) -> Self {
    self.compilation_options = compilation_options;
    self
  }

  #[inline]
  pub fn buffer_layouts(mut self, buffer_layouts: &'a [VertexBufferLayout<'a>]) -> Self {
    self.buffer_layouts = buffer_layouts;
    self
  }

  #[inline]
  pub fn build(self) -> Option<VertexState<'a>> { self.into() }
}

impl<'a> From<VertexState<'a>> for VertexStateBuilder<'a> {
  #[inline]
  fn from(vertex_state: VertexState<'a>) -> Self {
    Self {
      module: Some(vertex_state.module),
      entry_point: vertex_state.entry_point,
      compilation_options: vertex_state.compilation_options,
      buffer_layouts: vertex_state.buffers,
    }
  }
}

impl<'a> From<VertexStateBuilder<'a>> for Option<VertexState<'a>> {
  fn from(builder: VertexStateBuilder<'a>) -> Self {
    builder.module.map(|module| VertexState {
      module,
      entry_point: builder.entry_point,
      compilation_options: builder.compilation_options,
      buffers: builder.buffer_layouts,
    })
  }
}
