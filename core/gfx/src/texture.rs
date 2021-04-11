use image::RgbaImage;
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, Device, Extent3d, Origin3d, Queue, ShaderStage, Texture, TextureCopyView, TextureDataLayout, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage, TextureView, TextureViewDescriptor};

use common::screen::PhysicalSize;

use crate::bind_group::{BindGroupEntryBuilder, BindGroupLayoutEntryBuilder};
use crate::swap_chain::GfxSwapChain;

// Texture builder creation and modification

pub struct TextureBuilder<'a> {
  texture_descriptor: TextureDescriptor<'a>,
  texture_view_descriptor: TextureViewDescriptor<'a>,
}

impl<'a> TextureBuilder<'a> {
  #[inline]
  pub fn new() -> Self {
    Self {
      texture_descriptor: TextureDescriptor {
        size: Extent3d::default(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        label: None,
      },
      texture_view_descriptor: TextureViewDescriptor::default(),
    }
  }

  #[inline]
  pub fn new_from_2d_rgba_image(image: &RgbaImage) -> Self {
    let (width, height) = image.dimensions();
    Self::new()
      .with_2d_size(width, height)
      .with_rgba8_unorm_srgb_format()
      .with_sampled_usage()
  }

  #[inline]
  pub fn new_depth_32_float(size: PhysicalSize) -> Self {
    Self::new()
      .with_2d_size(size.width as u32, size.height as u32)
      .with_depth32_float_format()
      .with_render_attachment_usage()
  }

  #[inline]
  pub fn new_multisampled_framebuffer(swap_chain: &GfxSwapChain, sample_count: u32) -> Self {
    let (width, height) = swap_chain.get_size();
    Self::new()
      .with_2d_size(width, height)
      .with_sample_count(sample_count)
      .with_format(swap_chain.get_texture_format())
      .with_render_attachment_usage()
  }


  #[inline]
  pub fn with_size(mut self, size: Extent3d) -> Self {
    self.texture_descriptor.size = size;
    self
  }

  #[inline]
  pub fn with_mip_level_count(mut self, mip_level_count: u32) -> Self {
    self.texture_descriptor.mip_level_count = mip_level_count;
    self
  }

  #[inline]
  pub fn with_sample_count(mut self, sample_count: u32) -> Self {
    self.texture_descriptor.sample_count = sample_count;
    self
  }

  #[inline]
  pub fn with_dimension(mut self, dimension: TextureDimension) -> Self {
    self.texture_descriptor.dimension = dimension;
    self
  }

  #[inline]
  pub fn with_2d_size(self, width: u32, height: u32) -> Self {
    self
      .with_size(Extent3d { width, height, depth: 1 })
      .with_dimension(TextureDimension::D2)
  }

  #[inline]
  pub fn with_format(mut self, format: TextureFormat) -> Self {
    self.texture_descriptor.format = format;
    self
  }

  #[inline]
  pub fn with_rgba8_unorm_srgb_format(self) -> Self {
    self.with_format(TextureFormat::Rgba8UnormSrgb)
  }

  #[inline]
  pub fn with_depth32_float_format(self) -> Self {
    self.with_format(TextureFormat::Depth32Float)
  }

  #[inline]
  pub fn with_usage(mut self, usage: TextureUsage) -> Self {
    self.texture_descriptor.usage = usage;
    self
  }

  #[inline]
  pub fn with_sampled_usage(self) -> Self {
    self.with_usage(TextureUsage::SAMPLED | TextureUsage::COPY_DST)
  }

  #[inline]
  pub fn with_static_sampled_usage(self) -> Self {
    self.with_usage(TextureUsage::SAMPLED)
  }

  #[inline]
  pub fn with_render_attachment_usage(self) -> Self {
    self.with_usage(TextureUsage::RENDER_ATTACHMENT)
  }

  #[inline]
  pub fn with_texture_label(mut self, label: &'a str) -> Self {
    self.texture_descriptor.label = Some(label);
    self
  }

  #[inline]
  pub fn with_texture_view_label(mut self, label: &'a str) -> Self {
    self.texture_view_descriptor.label = Some(label);
    self
  }
}

// Texture creation

pub struct GfxTexture {
  pub texture: Texture,
  pub view: TextureView,
  pub size: Extent3d,
  pub format: TextureFormat,
}

impl<'a> TextureBuilder<'a> {
  #[inline]
  pub fn build(self, device: &Device) -> GfxTexture {
    let texture = device.create_texture(&self.texture_descriptor);
    let view = texture.create_view(&self.texture_view_descriptor);
    GfxTexture { texture, view, size: self.texture_descriptor.size, format: self.texture_descriptor.format }
  }
}

// Writing texture data

impl<'a> GfxTexture {
  #[inline]
  pub fn write_texture_data(&self, queue: &Queue, data: &[u8], bytes_per_row: u32, rows_per_image: u32) {
    queue.write_texture(
      TextureCopyView {
        texture: &self.texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
      },
      data,
      TextureDataLayout {
        offset: 0,
        bytes_per_row,
        rows_per_image,
      },
      self.size,
    );
  }

  #[inline]
  pub fn write_rgba_texture_data(&self, queue: &Queue, data: &[u8]) {
    self.write_texture_data(queue, data, 4 * self.size.width, self.size.height);
  }

  #[inline]
  pub fn write_2d_rgba_image(&self, queue: &Queue, image: RgbaImage) {
    self.write_rgba_texture_data(queue, image.as_raw());
  }
}

// Bind group (layout) entries creation

impl<'a> GfxTexture {
  #[inline]
  pub fn create_default_float_2d_bind_group_layout_entry(&self, binding_index: u32, shader_visibility: ShaderStage) -> BindGroupLayoutEntry {
    BindGroupLayoutEntryBuilder::new_default_float_2d_texture()
      .with_binding(binding_index)
      .with_shader_visibility(shader_visibility)
      .build()
  }

  #[inline]
  pub fn create_bind_group_entry(&'a self, binding_index: u32) -> BindGroupEntry<'a> {
    BindGroupEntryBuilder::new_texture_view(&self.view)
      .with_binding(binding_index)
      .build()
  }

  #[inline]
  pub fn create_default_float_2d_bind_group_entries(
    &'a self,
    binding_index: u32,
    shader_visibility: ShaderStage,
  ) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let bind_group_layout = self.create_default_float_2d_bind_group_layout_entry(binding_index, shader_visibility);
    let bind_group = self.create_bind_group_entry(binding_index);
    (bind_group_layout, bind_group)
  }
}
