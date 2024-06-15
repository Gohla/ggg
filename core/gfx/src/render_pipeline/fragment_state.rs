use wgpu::{ColorTargetState, FragmentState, PipelineCompilationOptions, ShaderModule};

pub struct FragmentStateBuilder<'a> {
  module: Option<&'a ShaderModule>,
  entry_point: &'a str,
  compilation_options: PipelineCompilationOptions<'a>,
  targets: &'a [Option<ColorTargetState>],
}

impl<'a> Default for FragmentStateBuilder<'a> {
  #[inline]
  fn default() -> Self {
    Self {
      module: None,
      entry_point: "main",
      compilation_options: PipelineCompilationOptions::default(),
      targets: &[],
    }
  }
}

impl<'a> FragmentStateBuilder<'a> {
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
  pub fn targets(mut self, targets: &'a [Option<ColorTargetState>]) -> Self {
    self.targets = targets;
    self
  }
}

impl<'a> From<FragmentState<'a>> for FragmentStateBuilder<'a> {
  #[inline]
  fn from(fragment_state: FragmentState<'a>) -> Self {
    Self {
      module: Some(fragment_state.module),
      entry_point: fragment_state.entry_point,
      compilation_options: fragment_state.compilation_options,
      targets: fragment_state.targets,
    }
  }
}
impl<'a> From<FragmentStateBuilder<'a>> for Option<FragmentState<'a>> {
  fn from(builder: FragmentStateBuilder<'a>) -> Self {
    builder.module.map(|module| FragmentState {
      module,
      entry_point: builder.entry_point,
      compilation_options: builder.compilation_options,
      targets: builder.targets,
    })
  }
}
