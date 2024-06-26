use image::{DynamicImage, GenericImageView};
use thiserror::Error;
use wgpu::{Device, Extent3d, Queue, ShaderStages};

use common::idx_assigner;
use common::idx_assigner::IdxAssigner;

use crate::bind_group::{CombinedBindGroup, CombinedBindGroupBuilder};
use crate::sampler::{GfxSampler, SamplerBuilder};
use crate::texture::{GfxTexture, TextureBuilder};

// Creation

pub struct ArrayTextureDefBuilder {
  width: u32,
  height: u32,
  assigner: IdxAssigner<TextureIdx, u16>,
  data: Vec<DynamicImage>,
}

impl ArrayTextureDefBuilder {
  pub fn new(width: u32, height: u32) -> Self {
    Self { width, height, assigner: IdxAssigner::new(), data: Vec::new() }
  }
}

// Adding a texture

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TextureIdx(u16);

#[derive(Error, Debug)]
pub enum AddTextureError {
  #[error("Adding a texture failed because the dimensions of the texture ({width}, {height}) are larger than the \
  maximum dimensions of the texture array ({max_width}, {max_height})")]
  IncorrectDimensionsFail { width: u32, height: u32, max_width: u32, max_height: u32 },
}

impl ArrayTextureDefBuilder {
  pub fn add_texture(&mut self, data: DynamicImage) -> Result<TextureIdx, AddTextureError> {
    let (width, height) = data.dimensions();
    if self.width < width || self.height < height {
      return Err(AddTextureError::IncorrectDimensionsFail { width, height, max_width: self.width, max_height: self.height });
    }
    let idx = self.assigner.assign_item();
    self.data.push(data);
    Ok(idx)
  }
}

// Building the texture array

impl ArrayTextureDefBuilder {
  pub fn build<'a>(
    self,
    device: &Device,
    queue: &Queue,
    texture_label: &'a str,
    texture_view_label: &'a str,
    sampler_label: &'a str,
    bind_group_layout_label: &'a str,
    bind_group_label: &'a str,
  ) -> ArrayTextureDef {
    let texture = TextureBuilder::new()
      .with_2d_array_size(self.width, self.height, self.data.len() as u32)
      .with_rgba8_unorm_srgb_format()
      .with_texture_label(texture_label)
      .with_texture_view_label(texture_view_label)
      .build(device);
    let texture_binding = texture.binding(0, ShaderStages::FRAGMENT);
    for (idx, data) in self.data.into_iter().enumerate() {
      let data = data.into_rgba8();
      let (width, height) = data.dimensions();
      texture.write_texture_data(queue, data.as_raw(), 0, Some(width * 4), None, Extent3d { width, height, depth_or_array_layers: idx as u32 });
    }
    let sampler = SamplerBuilder::new()
      .label(sampler_label)
      .build(device);
    let sampler_binding = sampler.binding(1, ShaderStages::FRAGMENT);
    let bind_group = CombinedBindGroupBuilder::new()
      .layout_entries(&[texture_binding.layout, sampler_binding.layout])
      .entries(&[texture_binding.entry, sampler_binding.entry])
      .layout_label(bind_group_layout_label)
      .label(bind_group_label)
      .build(device);
    ArrayTextureDef {
      texture,
      sampler,
      bind_group,
    }
  }
}

// Texture definition

pub struct ArrayTextureDef {
  pub texture: GfxTexture,
  pub sampler: GfxSampler,
  pub bind_group: CombinedBindGroup,
}

// Implementations

impl idx_assigner::Item for TextureIdx {
  type Idx = u16;

  #[inline]
  fn new(index: Self::Idx) -> Self {
    Self(index)
  }

  #[inline]
  fn into_idx(self) -> Self::Idx {
    self.0
  }
}
