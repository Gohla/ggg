use image::RgbaImage;
use wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType, Device, Extent3d, Queue, Sampler, SamplerDescriptor, ShaderStage, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsage, TextureView, TextureViewDescriptor, TextureViewDimension};

pub struct Texture2dRgbaBuilder<'a> {
  image: RgbaImage,
  size: Extent3d,
  texture_descriptor: TextureDescriptor<'a>,
  texture_view_descriptor: TextureViewDescriptor<'a>,
  sampler_descriptor: SamplerDescriptor<'a>,
  texture_view_binding_index: u32,
  sampler_binding_index: u32,
  shader_visibility: ShaderStage,
}

pub struct Texture2dRgba {
  pub texture: Texture,
  pub texture_view: TextureView,
  pub sampler: Sampler,
  texture_view_binding_index: u32,
  sampler_binding_index: u32,
  shader_visibility: ShaderStage,
}

impl<'a> Texture2dRgbaBuilder<'a> {
  pub fn new(image: RgbaImage) -> Self {
    let (width, height) = image.dimensions();
    let size = Extent3d { width, height, depth: 1 };
    Self {
      image,
      size,
      texture_descriptor: TextureDescriptor {
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        label: None,
      },
      texture_view_descriptor: TextureViewDescriptor::default(),
      sampler_descriptor: SamplerDescriptor::default(),
      texture_view_binding_index: 0,
      sampler_binding_index: 1,
      shader_visibility: ShaderStage::FRAGMENT,
    }
  }


  pub fn build(self, device: &Device, queue: &Queue) -> Texture2dRgba {
    let texture = device.create_texture(&self.texture_descriptor);
    queue.write_texture(
      wgpu::TextureCopyView {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
      },
      self.image.as_raw(),
      wgpu::TextureDataLayout {
        offset: 0,
        bytes_per_row: 4 * self.size.width,
        rows_per_image: self.size.height,
      },
      self.size,
    );
    let texture_view = texture.create_view(&self.texture_view_descriptor);
    let sampler = device.create_sampler(&self.sampler_descriptor);
    Texture2dRgba {
      texture,
      texture_view,
      sampler,
      texture_view_binding_index: self.texture_view_binding_index,
      sampler_binding_index: self.sampler_binding_index,
      shader_visibility: self.shader_visibility,
    }
  }

  pub fn build_with_default_bind_group(self, device: &Device, queue: &Queue) -> (Texture2dRgba, BindGroupLayout, BindGroup) {
    let texture = self.build(device, queue);
    let (bind_group_layout, bind_group) = texture.create_bindings().create_default_bind_group(device);
    (texture, bind_group_layout, bind_group)
  }
}


pub struct Texture2dRgbaBindings<'a> {
  pub texture_view_bind_group_layout_entry: BindGroupLayoutEntry,
  pub sampler_bind_group_layout_entry: BindGroupLayoutEntry,
  pub texture_view_bind_group_entry: BindGroupEntry<'a>,
  pub sampler_bind_group_entry: BindGroupEntry<'a>,
}

impl<'a> Texture2dRgba {
  pub fn create_bindings(&'a self) -> Texture2dRgbaBindings<'a> {
    let texture_view_bind_group_layout_entry = BindGroupLayoutEntry {
      binding: self.texture_view_binding_index,
      visibility: self.shader_visibility,
      ty: BindingType::Texture {
        multisampled: false,
        view_dimension: TextureViewDimension::D2,
        sample_type: TextureSampleType::Float { filterable: false },
      },
      count: None,
    };
    let sampler_bind_group_layout_entry = BindGroupLayoutEntry {
      binding: self.sampler_binding_index,
      visibility: self.shader_visibility,
      ty: wgpu::BindingType::Sampler {
        comparison: false,
        filtering: true,
      },
      count: None,
    };
    let texture_view_bind_group_entry = BindGroupEntry {
      binding: self.texture_view_binding_index,
      resource: BindingResource::TextureView(&self.texture_view),
    };
    let sampler_bind_group_entry = BindGroupEntry {
      binding: self.sampler_binding_index,
      resource: BindingResource::Sampler(&self.sampler),
    };
    Texture2dRgbaBindings {
      texture_view_bind_group_layout_entry,
      sampler_bind_group_layout_entry,
      texture_view_bind_group_entry,
      sampler_bind_group_entry,
    }
  }
}


impl<'a> Texture2dRgbaBindings<'a> {
  pub fn create_default_bind_group(self, device: &Device) -> (BindGroupLayout, BindGroup) {
    let bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        entries: &[
          self.texture_view_bind_group_layout_entry,
          self.sampler_bind_group_layout_entry,
        ],
        label: None,
      }
    );
    let bind_group = device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
          self.texture_view_bind_group_entry,
          self.sampler_bind_group_entry,
        ],
        label: None,
      }
    );
    (bind_group_layout, bind_group)
  }
}
