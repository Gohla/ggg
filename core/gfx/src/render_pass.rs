use wgpu::{Color, CommandEncoder, LoadOp, Operations, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, TextureView};

use crate::{Render, Gfx};

pub struct RenderPassBuilder<'a, 'b> {
  pub label: Option<&'a str>,
  pub color_attachments: &'b [Option<RenderPassColorAttachment<'a>>],
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
  pub fn with_color_attachments(mut self, color_attachments: &'b [Option<RenderPassColorAttachment<'a>>]) -> Self {
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
        load: LoadOp::Clear(0.0), // Using "reversed Z", so clearing depth to 0 instead of 1.
        store: StoreOp::Store,
      }),
      stencil_ops: None,
    })
  }

  #[inline]
  pub fn begin_render_pass(self, encoder: &'a mut CommandEncoder) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
      label: self.label,
      color_attachments: self.color_attachments,
      depth_stencil_attachment: self.depth_stencil_attachment,
      ..RenderPassDescriptor::default()
    })
  }


  /// Ignores the previously set `color_attachments`.
  #[inline]
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
        Some(RenderPassColorAttachment {
          view,
          resolve_target,
          ops,
        })
      ],
      depth_stencil_attachment: self.depth_stencil_attachment,
      ..RenderPassDescriptor::default()
    })
  }

  /// Ignores the previously set `color_attachments`.
  #[inline]
  pub fn begin_render_pass_for_swap_chain_with_clear(self, encoder: &'a mut CommandEncoder, framebuffer: &'a TextureView) -> RenderPass<'a> {
    self.begin_render_pass_with_color_attachment(encoder, framebuffer, None, Operations::default())
  }

  /// Ignores the previously set `color_attachments`.
  #[inline]
  pub fn begin_render_pass_for_swap_chain_with_load(self, encoder: &'a mut CommandEncoder, framebuffer: &'a TextureView) -> RenderPass<'a> {
    self.begin_render_pass_with_color_attachment(encoder, &framebuffer, None, Operations { load: LoadOp::Load, store: StoreOp::Store })
  }

  /// Ignores the previously set `color_attachments`.
  #[inline]
  pub fn begin_render_pass_for_multisampled_swap_chain_with_clear(self, encoder: &'a mut CommandEncoder, multisampled_framebuffer: &'a TextureView, framebuffer: &'a TextureView) -> RenderPass<'a> {
    self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(&framebuffer), Operations::default())
  }

  /// Ignores the previously set `color_attachments`.
  #[inline]
  pub fn begin_render_pass_for_multisampled_swap_chain_with_load(self, encoder: &'a mut CommandEncoder, multisampled_framebuffer: &'a TextureView, framebuffer: &'a TextureView) -> RenderPass<'a> {
    self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(&framebuffer), Operations { load: LoadOp::Load, store: StoreOp::Store })
  }

  /// Ignores the previously set `color_attachments`.
  #[inline]
  pub fn begin_render_pass_for_possibly_multisampled_swap_chain(self, encoder: &'a mut CommandEncoder, multisampled_framebuffer: Option<&'a TextureView>, framebuffer: &'a TextureView, ops: Operations<Color>) -> RenderPass<'a> {
    if let Some(multisampled_framebuffer) = multisampled_framebuffer {
      self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(framebuffer), ops)
    } else {
      self.begin_render_pass_with_color_attachment(encoder, framebuffer, None, ops)
    }
  }


  /// Ignores the previously set `color_attachments` and `depth_stencil_attachment` if `gfx` has a `depth_texture`.
  #[inline]
  pub fn begin_render_pass_for_gfx_frame(self, gfx: &'a Gfx, frame: &'a mut Render, attach_depth_stencil: bool, ops: Operations<Color>) -> RenderPass<'a> {
    let builder = match (attach_depth_stencil, &gfx.depth_stencil_texture) {
      (true, Some(depth_texture)) => self.with_depth_texture(&depth_texture.view),
      _ => self,
    };
    builder.begin_render_pass_for_possibly_multisampled_swap_chain(frame.encoder, gfx.multisampled_framebuffer.as_ref().map(|t| &t.view), frame.output_texture, ops)
  }

  /// Ignores the previously set `color_attachments` and `depth_stencil_attachment` if `gfx` has a `depth_texture`.
  #[inline]
  pub fn begin_render_pass_for_gfx_frame_simple(self, gfx: &'a Gfx, frame: &'a mut Render, attach_depth_stencil: bool, clear: bool) -> RenderPass<'a> {
    let ops = if clear {
      Operations::default()
    } else {
      Operations { load: LoadOp::Load, store: StoreOp::Store }
    };
    self.begin_render_pass_for_gfx_frame(gfx, frame, attach_depth_stencil, ops)
  }

  /// Ignores the previously set `color_attachments` and `depth_stencil_attachment` if `gfx` has a `depth_texture`.
  #[inline]
  pub fn begin_render_pass_for_gfx_frame_with_clear(self, gfx: &'a Gfx, frame: &'a mut Render, attach_depth_stencil: bool) -> RenderPass<'a> {
    self.begin_render_pass_for_gfx_frame(gfx, frame, attach_depth_stencil, Operations::default())
  }

  /// Ignores the previously set `color_attachments` and `depth_stencil_attachment` if `gfx` has a `depth_texture`.
  #[inline]
  pub fn begin_render_pass_for_gfx_frame_with_load(self, gfx: &'a Gfx, frame: &'a mut Render, attach_depth_stencil: bool) -> RenderPass<'a> {
    self.begin_render_pass_for_gfx_frame(gfx, frame, attach_depth_stencil, Operations { load: LoadOp::Load, store: StoreOp::Store })
  }
}
