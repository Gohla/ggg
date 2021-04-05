use wgpu::{CommandEncoder, LoadOp, Operations, RenderPass, RenderPassColorAttachmentDescriptor, RenderPassDepthStencilAttachmentDescriptor, RenderPassDescriptor, SwapChainTexture, TextureView};

pub struct RenderPassBuilder<'a, 'b> {
  pub label: Option<&'b str>,
  pub color_attachments: &'b [RenderPassColorAttachmentDescriptor<'a>],
  pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDescriptor<'a>>,
}

impl<'a, 'b> RenderPassBuilder<'a, 'b> {
  pub fn new() -> Self {
    Self {
      label: None,
      color_attachments: &[],
      depth_stencil_attachment: None,
    }
  }


  #[inline]
  pub fn with_label(mut self, label: &'a str) -> Self {
    self.label = Some(label);
    self
  }

  #[inline]
  pub fn with_color_attachments(mut self, color_attachments: &'b [RenderPassColorAttachmentDescriptor<'a>]) -> Self {
    self.color_attachments = color_attachments;
    self
  }

  #[inline]
  pub fn with_depth_stencil_attachment(mut self, depth_stencil_attachment: RenderPassDepthStencilAttachmentDescriptor<'a>) -> Self {
    self.depth_stencil_attachment = Some(depth_stencil_attachment);
    self
  }

  #[inline]
  pub fn with_depth_texture(self, depth_texture_view: &'a TextureView) -> Self {
    self.with_depth_stencil_attachment(RenderPassDepthStencilAttachmentDescriptor {
      attachment: depth_texture_view,
      depth_ops: Some(Operations {
        load: LoadOp::Clear(1.0),
        store: true,
      }),
      stencil_ops: None,
    })
  }

  pub fn begin_render_pass(self, encoder: &'a mut CommandEncoder) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
      label: self.label,
      color_attachments: self.color_attachments,
      depth_stencil_attachment: self.depth_stencil_attachment,
    })
  }

  pub fn begin_render_pass_for_swap_chain(self, encoder: &'a mut CommandEncoder, swap_chain_texture: &'a SwapChainTexture) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
      label: self.label,
      color_attachments: &[
        RenderPassColorAttachmentDescriptor {
          attachment: &swap_chain_texture.view,
          resolve_target: None,
          ops: Default::default(),
        }
      ],
      depth_stencil_attachment: self.depth_stencil_attachment,
    })
  }
}
