use image::RgbaImage;
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BufferAddress, Device, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, ShaderStages, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension};

use common::screen::PhysicalSize;

use crate::bind_group::{BindGroupEntryBuilder, BindGroupLayoutEntryBuilder};
use crate::surface::GfxSurface;

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
        label: None,
        size: Extent3d::default(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
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
  pub fn new_depth(size: PhysicalSize, format: TextureFormat) -> Self {
    Self::new()
      .with_2d_size(size.width as u32, size.height as u32)
      .with_format(format)
      .with_render_attachment_usage()
  }

  #[inline]
  pub fn new_depth_32_float(size: PhysicalSize) -> Self {
    Self::new_depth(size, TextureFormat::Depth32Float)
  }

  #[inline]
  pub fn new_multisampled_framebuffer(surface: &GfxSurface, sample_count: u32) -> Self {
    let (width, height): (u64, u64) = surface.get_size().physical.into();
    Self::new()
      .with_2d_size(width as u32, height as u32)
      .with_sample_count(sample_count)
      .with_format(surface.get_texture_format())
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
      .with_size(Extent3d { width, height, depth_or_array_layers: 1 })
      .with_dimension(TextureDimension::D2)
  }

  #[inline]
  pub fn with_2d_array_size(self, width: u32, height: u32, layer_count: u32) -> Self {
    self
      .with_size(Extent3d { width, height, depth_or_array_layers: layer_count })
      .with_dimension(TextureDimension::D2)
      .with_view_dimension(TextureViewDimension::D2Array)
      .with_view_array_layer_count(layer_count)
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
  pub fn with_usage(mut self, usage: TextureUsages) -> Self {
    self.texture_descriptor.usage = usage;
    self
  }

  #[inline]
  pub fn with_sampled_usage(self) -> Self {
    self.with_usage(TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST)
  }

  #[inline]
  pub fn with_static_sampled_usage(self) -> Self {
    self.with_usage(TextureUsages::TEXTURE_BINDING)
  }

  #[inline]
  pub fn with_render_attachment_usage(self) -> Self {
    self.with_usage(TextureUsages::RENDER_ATTACHMENT)
  }

  #[inline]
  pub fn with_texture_label(mut self, label: &'a str) -> Self {
    self.texture_descriptor.label = Some(label);
    self
  }


  #[inline]
  pub fn with_view_dimension(mut self, dimension: TextureViewDimension) -> Self {
    self.texture_view_descriptor.dimension = Some(dimension);
    self
  }

  #[inline]
  pub fn with_view_array_layer_count(mut self, array_layer_count: u32) -> Self {
    self.texture_view_descriptor.array_layer_count = Some(array_layer_count);
    self
  }

  #[inline]
  pub fn with_texture_view_label(mut self, label: &'a str) -> Self {
    self.texture_view_descriptor.label = Some(label);
    self
  }
}

// Texture creation

#[derive(Debug)]
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
  pub fn write_texture_data(&self, queue: &Queue, data: &[u8], offset: BufferAddress, bytes_per_row: Option<u32>, rows_per_image: Option<u32>, size: Extent3d) {
    queue.write_texture(
      ImageCopyTexture {
        texture: &self.texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
        aspect: TextureAspect::All,
      },
      data,
      ImageDataLayout {
        offset,
        bytes_per_row,
        rows_per_image,
      },
      size,
    );
  }

  #[inline]
  pub fn write_whole_texture_data(&self, queue: &Queue, data: &[u8], bytes_per_row: Option<u32>, rows_per_image: Option<u32>) {
    self.write_texture_data(queue, data, 0, bytes_per_row, rows_per_image, self.size);
  }

  #[inline]
  pub fn write_2d_rgba_texture_data(&self, queue: &Queue, data: &[u8]) {
    self.write_whole_texture_data(queue, data, Some(4 * self.size.width), None);
  }

  #[inline]
  pub fn write_2d_rgba_image(&self, queue: &Queue, image: RgbaImage) {
    self.write_2d_rgba_texture_data(queue, image.as_raw());
  }
}

// Bind group (layout) entries creation

impl<'a> GfxTexture {
  #[inline]
  pub fn create_default_float_2d_bind_group_layout_entry(&self, binding_index: u32, shader_visibility: ShaderStages) -> BindGroupLayoutEntry {
    BindGroupLayoutEntryBuilder::new_default_float_2d_texture()
      .with_binding(binding_index)
      .with_shader_visibility(shader_visibility)
      .build()
  }

  #[inline]
  pub fn create_default_float_2d_array_bind_group_layout_entry(&self, binding_index: u32, shader_visibility: ShaderStages) -> BindGroupLayoutEntry {
    BindGroupLayoutEntryBuilder::new_default_float_2d_array_texture()
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
    shader_visibility: ShaderStages,
  ) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let bind_group_layout = self.create_default_float_2d_bind_group_layout_entry(binding_index, shader_visibility);
    let bind_group = self.create_bind_group_entry(binding_index);
    (bind_group_layout, bind_group)
  }

  #[inline]
  pub fn create_default_float_2d_array_bind_group_entries(
    &'a self,
    binding_index: u32,
    shader_visibility: ShaderStages,
  ) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let bind_group_layout = self.create_default_float_2d_array_bind_group_layout_entry(binding_index, shader_visibility);
    let bind_group = self.create_bind_group_entry(binding_index);
    (bind_group_layout, bind_group)
  }
}
