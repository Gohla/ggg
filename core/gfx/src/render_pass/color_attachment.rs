use wgpu::{Color, LoadOp, Operations, RenderPassColorAttachment, StoreOp, TextureView};

#[derive(Default, Copy, Clone, Debug)]
pub struct ColorAttachmentBuilder<'pass> {
  view: Option<&'pass TextureView>,
  resolve_target: Option<&'pass TextureView>,
  ops: Operations<Color>,
}

impl<'pass> ColorAttachmentBuilder<'pass> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[inline]
  pub fn view(mut self, view: &'pass TextureView) -> Self {
    self.view = Some(view);
    self
  }
  #[inline]
  pub fn without_view(mut self) -> Self {
    self.view = None;
    self
  }

  #[inline]
  pub fn resolve_target(mut self, resolve_target: &'pass TextureView) -> Self {
    self.resolve_target = Some(resolve_target);
    self
  }
  #[inline]
  pub fn without_resolve_target(mut self) -> Self {
    self.resolve_target = None;
    self
  }

  #[inline]
  pub fn maybe_multisample(self, view: &'pass TextureView, multisample_view: Option<&'pass TextureView>) -> Self {
    if let Some(multisample_view) = multisample_view {
      self
        .view(multisample_view)
        .resolve_target(view)
    } else {
      self.view(view)
    }
  }

  #[inline]
  pub fn ops(mut self, ops: Operations<Color>) -> Self {
    self.ops = ops;
    self
  }

  #[inline]
  pub fn load_op(mut self, load: LoadOp<Color>) -> Self {
    self.ops.load = load;
    self
  }
  #[inline]
  pub fn load(self) -> Self {
    self.load_op(LoadOp::Load)
  }
  #[inline]
  pub fn clear(self, value: Color) -> Self {
    self.load_op(LoadOp::Clear(value))
  }
  #[inline]
  pub fn clear_default(self) -> Self {
    self.clear(Color::default())
  }
  #[inline]
  pub fn clear_or_load(self, clear_value: Option<Color>) -> Self {
    if let Some(value) = clear_value { self.clear(value) } else { self.load() }
  }
  #[inline]
  pub fn clear_default_or_load(self, clear: bool) -> Self {
    if clear { self.clear_default() } else { self.load() }
  }

  #[inline]
  pub fn store_op(mut self, store: StoreOp) -> Self {
    self.ops.store = store;
    self
  }
  #[inline]
  pub fn store(self) -> Self {
    self.store_op(StoreOp::Store)
  }
  #[inline]
  pub fn discard(self) -> Self {
    self.store_op(StoreOp::Discard)
  }

  #[inline]
  pub fn build(self) -> Option<RenderPassColorAttachment<'pass>> { self.into() }
}

impl<'pass> From<RenderPassColorAttachment<'pass>> for ColorAttachmentBuilder<'pass> {
  #[inline]
  fn from(color_attachment: RenderPassColorAttachment<'pass>) -> Self {
    Self {
      view: Some(color_attachment.view),
      resolve_target: color_attachment.resolve_target,
      ops: color_attachment.ops,
    }
  }
}

impl<'pass> From<ColorAttachmentBuilder<'pass>> for Option<RenderPassColorAttachment<'pass>> {
  fn from(builder: ColorAttachmentBuilder<'pass>) -> Self {
    builder.view.map(|view| RenderPassColorAttachment {
      view,
      resolve_target: builder.resolve_target,
      ops: builder.ops,
    })
  }
}
