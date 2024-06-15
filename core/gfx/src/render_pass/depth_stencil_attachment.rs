use wgpu::{LoadOp, Operations, RenderPassDepthStencilAttachment, StoreOp, TextureView};

#[derive(Default, Copy, Clone, Debug)]
pub struct DepthStencilAttachmentBuilder<'pass> {
  view: Option<&'pass TextureView>,
  depth_ops: Option<Operations<f32>>,
  stencil_ops: Option<Operations<u32>>,
}

impl<'pass> DepthStencilAttachmentBuilder<'pass> {
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
  pub fn depth_ops(mut self, depth_ops: Operations<f32>) -> Self {
    self.depth_ops = Some(depth_ops);
    self
  }
  #[inline]
  pub fn without_depth_ops(mut self) -> Self {
    self.depth_ops = None;
    self
  }
  #[inline]
  pub fn depth_ops_reverse_z_clear_store(self) -> Self {
    self.depth_clear_reverse_z().depth_store()
  }
  #[inline]
  pub fn depth_ops_load_store(self) -> Self {
    self.depth_load().depth_store()
  }

  #[inline]
  pub fn depth_load_op(mut self, load: LoadOp<f32>) -> Self {
    self.depth_ops.get_or_insert(Operations::default()).load = load;
    self
  }
  #[inline]
  pub fn depth_load(self) -> Self {
    self.depth_load_op(LoadOp::Load)
  }
  #[inline]
  pub fn depth_clear(self, value: f32) -> Self {
    self.depth_load_op(LoadOp::Clear(value))
  }
  #[inline]
  pub fn depth_clear_reverse_z(self) -> Self {
    // Using "reverse Z", so clearing depth to 0 instead of 1.
    self.depth_clear(0.0)
  }

  #[inline]
  pub fn depth_store_op(mut self, store: StoreOp) -> Self {
    self.depth_ops.get_or_insert(Operations::default()).store = store;
    self
  }
  #[inline]
  pub fn depth_store(self) -> Self {
    self.depth_store_op(StoreOp::Store)
  }
  #[inline]
  pub fn depth_discard(self) -> Self {
    self.depth_store_op(StoreOp::Discard)
  }

  #[inline]
  pub fn maybe_depth_reverse_z(self, view: Option<&'pass TextureView>) -> Self {
    if let Some(view) = view {
      self
        .view(view)
        .depth_clear_reverse_z()
        .depth_store()
    } else {
      self
    }
  }


  #[inline]
  pub fn stencil_ops(mut self, stencil_ops: Operations<u32>) -> Self {
    self.stencil_ops = Some(stencil_ops);
    self
  }
  #[inline]
  pub fn without_stencil_ops(mut self) -> Self {
    self.stencil_ops = None;
    self
  }

  #[inline]
  pub fn stencil_load_op(mut self, load: LoadOp<u32>) -> Self {
    self.stencil_ops.get_or_insert(Operations::default()).load = load;
    self
  }
  #[inline]
  pub fn stencil_load(self) -> Self {
    self.stencil_load_op(LoadOp::Load)
  }
  #[inline]
  pub fn stencil_clear(self, value: u32) -> Self {
    self.stencil_load_op(LoadOp::Clear(value))
  }

  #[inline]
  pub fn stencil_store_op(mut self, store: StoreOp) -> Self {
    self.stencil_ops.get_or_insert(Operations::default()).store = store;
    self
  }
  #[inline]
  pub fn stencil_store(self) -> Self {
    self.stencil_store_op(StoreOp::Store)
  }
  #[inline]
  pub fn stencil_discard(self) -> Self {
    self.stencil_store_op(StoreOp::Discard)
  }


  #[inline]
  pub fn build(self) -> Option<RenderPassDepthStencilAttachment<'pass>> { self.into() }
}

impl<'pass> From<RenderPassDepthStencilAttachment<'pass>> for DepthStencilAttachmentBuilder<'pass> {
  #[inline]
  fn from(depth_stencil_attachment: RenderPassDepthStencilAttachment<'pass>) -> Self {
    Self {
      view: Some(depth_stencil_attachment.view),
      depth_ops: depth_stencil_attachment.depth_ops,
      stencil_ops: depth_stencil_attachment.stencil_ops,
    }
  }
}

impl<'pass> From<DepthStencilAttachmentBuilder<'pass>> for Option<RenderPassDepthStencilAttachment<'pass>> {
  fn from(builder: DepthStencilAttachmentBuilder<'pass>) -> Self {
    builder.view.map(|view| RenderPassDepthStencilAttachment {
      view,
      depth_ops: builder.depth_ops,
      stencil_ops: builder.stencil_ops,
    })
  }
}
