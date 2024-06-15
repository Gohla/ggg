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
  pub fn depth_stencil_view(mut self, view: &'pass TextureView) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.view(view);
    self
  }
  #[inline]
  pub fn without_depth_stencil_view(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.without_view();
    self
  }

  #[inline]
  pub fn depth_clear_reverse_z(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.depth_clear_reverse_z();
    self
  }
  #[inline]
  pub fn depth_load(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.depth_load();
    self
  }

  #[inline]
  pub fn maybe_depth_reverse_z(mut self, view: Option<&'pass TextureView>) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.maybe_depth_reverse_z(view);
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
  pub fn resolve_target(mut self, resolve_target: &'pass TextureView) -> Self {
    self.color_attachment = self.color_attachment.resolve_target(resolve_target);
    self
  }
  #[inline]
  pub fn without_resolve_target(mut self) -> Self {
    self.color_attachment = self.color_attachment.without_resolve_target();
    self
  }

  #[inline]
  pub fn maybe_multisample(mut self, view: &'pass TextureView, multisample_view: Option<&'pass TextureView>) -> Self {
    self.color_attachment = self.color_attachment.maybe_multisample(view, multisample_view);
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
  pub fn depth_stencil_view(mut self, view: &'pass TextureView) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.view(view);
    self
  }
  #[inline]
  pub fn without_depth_stencil_view(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.without_view();
    self
  }

  #[inline]
  pub fn depth_clear_reverse_z(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.depth_clear_reverse_z();
    self
  }
  #[inline]
  pub fn depth_load(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.depth_load();
    self
  }

  #[inline]
  pub fn depth_store(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.depth_store();
    self
  }
  #[inline]
  pub fn depth_discard(mut self) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.depth_discard();
    self
  }

  #[inline]
  pub fn maybe_depth_reverse_z(mut self, view: Option<&'pass TextureView>) -> Self {
    self.depth_stencil_attachment = self.depth_stencil_attachment.maybe_depth_reverse_z(view);
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
