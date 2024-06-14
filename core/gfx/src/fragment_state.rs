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
  pub fn with_module(mut self, module: &'a ShaderModule) -> Self {
    self.module = Some(module);
    self
  }
  #[inline]
  pub fn set_module(&mut self, module: &'a ShaderModule) {
    self.module = Some(module);
  }

  #[inline]
  pub fn with_entry_point(mut self, entry_point: &'a str) -> Self {
    self.entry_point = entry_point;
    self
  }
  #[inline]
  pub fn set_entry_point(&mut self, entry_point: &'a str) {
    self.entry_point = entry_point;
  }

  #[inline]
  pub fn with_compilation_options(mut self, compilation_options: PipelineCompilationOptions<'a>) -> Self {
    self.compilation_options = compilation_options;
    self
  }
  #[inline]
  pub fn set_compilation_options(&mut self, compilation_options: PipelineCompilationOptions<'a>) {
    self.compilation_options = compilation_options;
  }

  #[inline]
  pub fn with_targets(mut self, targets: &'a [Option<ColorTargetState>]) -> Self {
    self.targets = targets;
    self
  }
  #[inline]
  pub fn set_targets(&mut self, targets: &'a [Option<ColorTargetState>]) {
    self.targets = targets;
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
