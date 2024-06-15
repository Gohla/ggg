use wgpu::{Color, CommandEncoder, Label, QuerySet, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPassTimestampWrites, TextureView};

use crate::render_pass::color_attachment::ColorAttachmentBuilder;
use crate::render_pass::depth_stencil_attachment::DepthStencilAttachmentBuilder;

pub mod color_attachment;
pub mod depth_stencil_attachment;

/// Builder for a [RenderPass]. Lifetime `'pass` lives as long as the render pass returned from [begin](Self::begin),
/// whereas `'build` only lives as long as this builder.
#[derive(Default, Clone, Debug)]
pub struct RenderPassBuilder<'pass, 'build> {
  label: Label<'build>,
  color_attachments: &'build [Option<RenderPassColorAttachment<'pass>>],
  depth_stencil_attachment: DepthStencilAttachmentBuilder<'pass>,
  timestamp_writes: Option<RenderPassTimestampWrites<'build>>,
  occlusion_query_set: Option<&'pass QuerySet>,
}

impl<'pass, 'build> RenderPassBuilder<'pass, 'build> {
  #[inline]
  pub fn new() -> Self { Self::default() }

  #[inline]
  pub fn label(mut self, label: &'build str) -> Self {
    self.label = Some(label);
    self
  }

  #[inline]
  pub fn color_attachments(mut self, color_attachments: &'build [Option<RenderPassColorAttachment<'pass>>]) -> Self {
    self.color_attachments = color_attachments;
    self
  }

  #[inline]
  pub fn depth_stencil_attachment(mut self, depth_stencil_attachment: RenderPassDepthStencilAttachment<'pass>) -> Self {
    self.depth_stencil_attachment = depth_stencil_attachment.into();
    self
  }
  #[inline]
  pub fn depth_texture(mut self, view: Option<&'pass TextureView>) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.depth_reverse_z(view);
    self
  }

  #[inline]
  pub fn timestamp_writes(mut self, timestamp_writes: RenderPassTimestampWrites<'build>) -> Self {
    self.timestamp_writes = Some(timestamp_writes);
    self
  }

  #[inline]
  pub fn occlusion_query_set(mut self, occlusion_query_set: &'pass QuerySet) -> Self {
    self.occlusion_query_set = Some(occlusion_query_set);
    self
  }

  #[inline]
  pub fn begin(self, encoder: &'pass mut CommandEncoder) -> RenderPass<'pass> {
    encoder.begin_render_pass(&RenderPassDescriptor {
      label: self.label,
      color_attachments: self.color_attachments,
      depth_stencil_attachment: self.depth_stencil_attachment.into(),
      timestamp_writes: self.timestamp_writes,
      occlusion_query_set: self.occlusion_query_set,
    })
  }
}

/// Builder for a [RenderPass] with a single color attachment. Lifetime `'pass` lives as long as the render pass
/// returned from [begin](Self::begin), whereas `'build` only lives as long as this builder.
#[derive(Debug)]
pub struct SingleRenderPassBuilder<'pass, 'build> {
  encoder: &'pass mut CommandEncoder,
  label: Label<'build>,
  color_attachment: ColorAttachmentBuilder<'pass>,
  depth_stencil_attachment: DepthStencilAttachmentBuilder<'pass>,
  timestamp_writes: Option<RenderPassTimestampWrites<'build>>,
  occlusion_query_set: Option<&'pass QuerySet>,
}

impl<'pass, 'build> SingleRenderPassBuilder<'pass, 'build> {
  #[inline]
  pub fn new(encoder: &'pass mut CommandEncoder) -> Self {
    Self {
      encoder,
      label: None,
      color_attachment: ColorAttachmentBuilder::default(),
      depth_stencil_attachment: DepthStencilAttachmentBuilder::default(),
      timestamp_writes: None,
      occlusion_query_set: None,
    }
  }

  #[inline]
  pub fn label(mut self, label: &'build str) -> Self {
    self.label = Some(label);
    self
  }

  #[inline]
  pub fn color_attachment(mut self, color_attachment: RenderPassColorAttachment<'pass>) -> Self {
    self.color_attachment = color_attachment.into();
    self
  }
  #[inline]
  pub fn modify_color_attachment(mut self, modify: impl FnOnce(ColorAttachmentBuilder<'pass>) -> ColorAttachmentBuilder<'pass>) -> Self {
    self.color_attachment = modify(self.color_attachment);
    self
  }
  #[inline]
  pub fn view(mut self, view: &'pass TextureView) -> Self {
    self.color_attachment = self.color_attachment.view(view);
    self
  }
  #[inline]
  pub fn without_view(mut self) -> Self {
    self.color_attachment = self.color_attachment.without_view();
    self
  }
  #[inline]
  pub fn resolve_target(mut self, resolve_target: Option<&'pass TextureView>) -> Self {
    self.color_attachment = self.color_attachment.resolve_target(resolve_target);
    self
  }
  #[inline]
  pub fn without_resolve_target(mut self) -> Self {
    self.color_attachment = self.color_attachment.without_resolve_target();
    self
  }
  #[inline]
  pub fn load(mut self) -> Self {
    self.color_attachment = self.color_attachment.load();
    self
  }
  #[inline]
  pub fn clear(mut self, value: Color) -> Self {
    self.color_attachment = self.color_attachment.clear(value);
    self
  }
  #[inline]
  pub fn clear_default(mut self) -> Self {
    self.color_attachment = self.color_attachment.clear_default();
    self
  }
  #[inline]
  pub fn clear_or_load(mut self, clear_value: Option<Color>) -> Self {
    self.color_attachment = self.color_attachment.clear_or_load(clear_value);
    self
  }
  #[inline]
  pub fn clear_default_or_load(mut self, clear: bool) -> Self {
    self.color_attachment = self.color_attachment.clear_default_or_load(clear);
    self
  }
  #[inline]
  pub fn store(mut self) -> Self {
    self.color_attachment = self.color_attachment.store();
    self
  }
  #[inline]
  pub fn discard(mut self) -> Self {
    self.color_attachment = self.color_attachment.discard();
    self
  }

  #[inline]
  pub fn depth_stencil_attachment(mut self, depth_stencil_attachment: RenderPassDepthStencilAttachment<'pass>) -> Self {
    self.depth_stencil_attachment = depth_stencil_attachment.into();
    self
  }
  #[inline]
  pub fn depth_reverse_z(mut self, view: Option<&'pass TextureView>) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.depth_reverse_z(view);
    self
  }
  #[inline]
  pub fn without_depth(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.without_depth();
    self
  }

  #[inline]
  pub fn timestamp_writes(mut self, timestamp_writes: RenderPassTimestampWrites<'build>) -> Self {
    self.timestamp_writes = Some(timestamp_writes);
    self
  }

  #[inline]
  pub fn occlusion_query_set(mut self, occlusion_query_set: &'pass QuerySet) -> Self {
    self.occlusion_query_set = Some(occlusion_query_set);
    self
  }

  #[inline]
  pub fn begin(self) -> RenderPass<'pass> {
    self.encoder.begin_render_pass(&RenderPassDescriptor {
      label: self.label,
      color_attachments: &[self.color_attachment.build()],
      depth_stencil_attachment: self.depth_stencil_attachment.into(),
      timestamp_writes: self.timestamp_writes,
      occlusion_query_set: self.occlusion_query_set,
    })
  }
}
//
// impl<'desc, 'tex> RenderPassBuilder<'desc, 'tex> {
//   /// Ignores the previously set `color_attachments`.
//   #[inline]
//   pub fn begin_render_pass_with_color_attachment(
//     self,
//     encoder: &'desc mut CommandEncoder,
//     view: &'desc TextureView,
//     resolve_target: Option<&'desc TextureView>,
//     ops: Operations<Color>,
//   ) -> RenderPass<'desc> {
//     encoder.begin_render_pass(&RenderPassDescriptor {
//       label: self.label,
//       color_attachments: &[
//         Some(RenderPassColorAttachment {
//           view,
//           resolve_target,
//           ops,
//         })
//       ],
//       depth_stencil_attachment: self.depth_stencil_attachment,
//       ..RenderPassDescriptor::default()
//     })
//   }
//
//   /// Ignores the previously set `color_attachments`.
//   #[inline]
//   pub fn begin_render_pass_for_swap_chain_with_clear(self, encoder: &'desc mut CommandEncoder, framebuffer: &'desc TextureView) -> RenderPass<'desc> {
//     self.begin_render_pass_with_color_attachment(encoder, framebuffer, None, Operations::default())
//   }
//
//   /// Ignores the previously set `color_attachments`.
//   #[inline]
//   pub fn begin_render_pass_for_swap_chain_with_load(self, encoder: &'desc mut CommandEncoder, framebuffer: &'desc TextureView) -> RenderPass<'desc> {
//     self.begin_render_pass_with_color_attachment(encoder, &framebuffer, None, Operations { load: LoadOp::Load, store: StoreOp::Store })
//   }
//
//   /// Ignores the previously set `color_attachments`.
//   #[inline]
//   pub fn begin_render_pass_for_multisampled_swap_chain_with_clear(self, encoder: &'desc mut CommandEncoder, multisampled_framebuffer: &'desc TextureView, framebuffer: &'desc TextureView) -> RenderPass<'desc> {
//     self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(&framebuffer), Operations::default())
//   }
//
//   /// Ignores the previously set `color_attachments`.
//   #[inline]
//   pub fn begin_render_pass_for_multisampled_swap_chain_with_load(self, encoder: &'desc mut CommandEncoder, multisampled_framebuffer: &'desc TextureView, framebuffer: &'desc TextureView) -> RenderPass<'desc> {
//     self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(&framebuffer), Operations { load: LoadOp::Load, store: StoreOp::Store })
//   }
//
//   /// Ignores the previously set `color_attachments`.
//   #[inline]
//   pub fn begin_render_pass_for_possibly_multisampled_swap_chain(self, encoder: &'desc mut CommandEncoder, multisampled_framebuffer: Option<&'desc TextureView>, framebuffer: &'desc TextureView, ops: Operations<Color>) -> RenderPass<'desc> {
//     if let Some(multisampled_framebuffer) = multisampled_framebuffer {
//       self.begin_render_pass_with_color_attachment(encoder, multisampled_framebuffer, Some(framebuffer), ops)
//     } else {
//       self.begin_render_pass_with_color_attachment(encoder, framebuffer, None, ops)
//     }
//   }
//
//
//   /// Ignores the previously set `color_attachments` and `depth_stencil_attachment` if `gfx` has a `depth_texture`.
//   #[inline]
//   pub fn begin_render_pass_for_gfx_frame(self, gfx: &'desc Gfx, frame: &'desc mut Render, attach_depth_stencil: bool, ops: Operations<Color>) -> RenderPass<'desc> {
//     let builder = match (attach_depth_stencil, &gfx.depth_stencil_texture) {
//       (true, Some(depth_texture)) => self.depth_texture(&depth_texture.view),
//       _ => self,
//     };
//     builder.begin_render_pass_for_possibly_multisampled_swap_chain(frame.encoder, gfx.multisampled_framebuffer.as_ref().map(|t| &t.view), frame.output_texture, ops)
//   }
//
//   /// Ignores the previously set `color_attachments` and `depth_stencil_attachment` if `gfx` has a `depth_texture`.
//   #[inline]
//   pub fn begin_render_pass_for_gfx_frame_simple(self, gfx: &'desc Gfx, frame: &'desc mut Render, attach_depth_stencil: bool, clear: bool) -> RenderPass<'desc> {
//     let ops = if clear {
//       Operations::default()
//     } else {
//       Operations { load: LoadOp::Load, store: StoreOp::Store }
//     };
//     self.begin_render_pass_for_gfx_frame(gfx, frame, attach_depth_stencil, ops)
//   }
//
//   /// Ignores the previously set `color_attachments` and `depth_stencil_attachment` if `gfx` has a `depth_texture`.
//   #[inline]
//   pub fn begin_render_pass_for_gfx_frame_with_clear(self, gfx: &'desc Gfx, frame: &'desc mut Render, attach_depth_stencil: bool) -> RenderPass<'desc> {
//     self.begin_render_pass_for_gfx_frame(gfx, frame, attach_depth_stencil, Operations::default())
//   }
//
//   /// Ignores the previously set `color_attachments` and `depth_stencil_attachment` if `gfx` has a `depth_texture`.
//   #[inline]
//   pub fn begin_render_pass_for_gfx_frame_with_load(self, gfx: &'desc Gfx, frame: &'desc mut Render, attach_depth_stencil: bool) -> RenderPass<'desc> {
//     self.begin_render_pass_for_gfx_frame(gfx, frame, attach_depth_stencil, Operations { load: LoadOp::Load, store: StoreOp::Store })
//   }
// }
