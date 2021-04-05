use image::RgbaImage;
use wgpu::{BindGroupEntry, BindGroupLayoutEntry, BindingResource, BindingType, Device, Extent3d, Origin3d, Queue, ShaderStage, Texture, TextureCopyView, TextureDataLayout, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsage, TextureView, TextureViewDescriptor, TextureViewDimension};

use common::screen::PhysicalSize;

// Texture builder creation and modification

pub struct TextureBuilder<'a> {
  texture_descriptor: TextureDescriptor<'a>,
  texture_view_descriptor: TextureViewDescriptor<'a>,
}

impl<'a> TextureBuilder<'a> {
  pub fn new(size: Extent3d, dimension: TextureDimension, format: TextureFormat, usage: TextureUsage) -> Self {
    Self {
      texture_descriptor: TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension,
        format,
        usage,
        label: None,
      },
      texture_view_descriptor: TextureViewDescriptor::default(),
    }
  }

  pub fn new_from_2d_rgba_image(image: &RgbaImage) -> Self {
    let (width, height) = image.dimensions();
    Self::new(
      Extent3d { width, height, depth: 1 },
      TextureDimension::D2,
      TextureFormat::Rgba8UnormSrgb,
      TextureUsage::SAMPLED | TextureUsage::COPY_DST,
    )
  }

  pub fn new_depth_32_float(size: PhysicalSize) -> Self {
    Self::new(
      Extent3d { width: size.width, height: size.height, depth: 1 },
      TextureDimension::D2,
      TextureFormat::Depth32Float,
      TextureUsage::RENDER_ATTACHMENT,
    )
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
  pub fn build(self, device: &Device) -> GfxTexture {
    let texture = device.create_texture(&self.texture_descriptor);
    let view = texture.create_view(&self.texture_view_descriptor);
    GfxTexture { texture, view, size: self.texture_descriptor.size, format: self.texture_descriptor.format }
  }
}

// Writing image data

impl<'a> GfxTexture {
  pub fn write_2d_rgba_image(&self, queue: &Queue, image: RgbaImage) {
    queue.write_texture(
      TextureCopyView {
        texture: &self.texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
      },
      image.as_raw(),
      TextureDataLayout {
        offset: 0,
        bytes_per_row: 4 * self.size.width,
        rows_per_image: self.size.height,
      },
      self.size,
    );
  }
}

// Bind group (layout) entries creation

impl<'a> GfxTexture {
  pub fn create_bind_group_entries(
    &'a self,
    binding_index: u32,
    shader_visibility: ShaderStage,
  ) -> (BindGroupLayoutEntry, BindGroupEntry<'a>) {
    let bind_group_layout = BindGroupLayoutEntry {
      binding: binding_index,
      visibility: shader_visibility,
      ty: BindingType::Texture {
        multisampled: false,
        view_dimension: TextureViewDimension::D2,
        sample_type: TextureSampleType::Float { filterable: false },
      },
      count: None,
    };
    let bind_group = BindGroupEntry {
      binding: binding_index,
      resource: BindingResource::TextureView(&self.view),
    };
    (bind_group_layout, bind_group)
  }
}
