use wgpu::{Color, CommandEncoder, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureView};

pub struct RenderPassBuilder<'a, 'b> {
  pub label: Option<&'a str>,
  pub color_attachments: &'b [RenderPassColorAttachment<'a>],
  pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment<'a>>,
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
  pub fn with_color_attachments(mut self, color_attachments: &'b [RenderPassColorAttachment<'a>]) -> Self {
    self.color_attachments = color_attachments;
    self
  }

  #[inline]
  pub fn with_depth_stencil_attachment(mut self, depth_stencil_attachment: RenderPassDepthStencilAttachment<'a>) -> Self {
    self.depth_stencil_attachment = Some(depth_stencil_attachment);
    self
  }

  #[inline]
  pub fn with_depth_texture(self, depth_texture_view: &'a TextureView) -> Self {
    self.with_depth_stencil_attachment(RenderPassDepthStencilAttachment {
      view: depth_texture_view,
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

  /// Ignores the previously set `color_attachments`.
  pub fn begin_render_pass_with_color_attachment(
    self,
    encoder: &'a mut CommandEncoder,
    view: &'a TextureView,
    resolve_target: Option<&'a TextureView>,
    ops: Operations<Color>,
  ) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
      label: self.label,
      color_attachments: &[
        RenderPassColorAttachment {
          view,
          resolve_target,
          ops,
        }
      ],
      depth_stencil_attachment: self.depth_stencil_attachment,
    })
  }

  /// Ignores the previously set `color_attachments`.
  pub fn begin_render_pass_for_swap_chain_with_clear(self, encoder: &'a mut CommandEncoder, framebuffer: &'a TextureView) -> RenderPass<'a> {
    self.begin_render_pass_with_color_attachment(encoder, framebuffer, None, Operations::default())
  }

  /// Ignores the previously set `color_attachments`.
  pub fn begin_render_pass_for_swap_chain_with_load(self, encoder: &'a mut CommandEncoder, framebuffer: &'a TextureView) -> RenderPass<'a> {
    self.begin_render_pass_with_color_attachment(encoder, &framebuffer, None, Operations { load: LoadOp::Load, store: true })
  }

  /// Ignores the previously set `color_attachments`.
  pub fn begin_render_pass_for_multisampled_swap_chain_with_clear(self, encoder: &'a mut CommandEncoder, multisampled_framebuffer: &'a TextureView, framebuffer: &'a TextureView) -> RenderPass<'a> {
    self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(&framebuffer), Operations::default())
  }

  /// Ignores the previously set `color_attachments`.
  pub fn begin_render_pass_for_multisampled_swap_chain_with_load(self, encoder: &'a mut CommandEncoder, multisampled_framebuffer: &'a TextureView, framebuffer: &'a TextureView) -> RenderPass<'a> {
    self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(&framebuffer), Operations { load: LoadOp::Load, store: true })
  }

  /// Ignores the previously set `color_attachments`.
  pub fn begin_render_pass_for_possibly_multisampled_swap_chain(self, encoder: &'a mut CommandEncoder, multisampled_framebuffer: Option<&'a TextureView>, framebuffer: &'a TextureView, ops: Operations<Color>) -> RenderPass<'a> {
    if let Some(multisampled_framebuffer) = multisampled_framebuffer {
      self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(framebuffer), ops)
    } else {
      self.begin_render_pass_with_color_attachment(encoder, framebuffer, None, ops)
    }
  }
}
