use std::num::NonZeroU32;

use image::{DynamicImage, GenericImageView};
use thiserror::Error;
use wgpu::{BindGroup, BindGroupLayout, Device, Extent3d, Queue, Sampler, ShaderStage};

use common::idx_assigner;
use common::idx_assigner::IdxAssigner;

use crate::bind_group::CombinedBindGroupLayoutBuilder;
use crate::sampler::{SamplerBuilder, SamplerBuilderSamplerEx};
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
  #[error("Adding a texture failed because the dimensions of the texture ({width}, {height}) are larger than the maximum dimensions of the texture array ({max_width}, {max_height})")]
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

#[derive(Error, Debug)]
pub enum BuildTextureArrayError {
  #[error("Building texture array failed because no textures were added")]
  NoTexturesFail,
}

impl ArrayTextureDefBuilder {
  pub fn build<'a>(self, device: &Device, queue: &Queue, texture_label: &'a str, texture_view_label: &'a str, sampler_label: &'a str, bind_group_layout_label: &'a str, bind_group_label: &'a str) -> Result<ArrayTextureDef, BuildTextureArrayError> {
    let layer_count = NonZeroU32::new(self.data.len() as u32).ok_or(BuildTextureArrayError::NoTexturesFail)?;
    let texture = TextureBuilder::new()
      .with_2d_array_size(self.width, self.height, layer_count)
      .with_rgba8_unorm_srgb_format()
      .with_texture_label(texture_label)
      .with_texture_view_label(texture_view_label)
      .build(device);
    let (texture_layout_entry, texture_bind_group_entry) = texture.create_default_float_2d_array_bind_group_entries(0, ShaderStage::FRAGMENT);
    for (idx, data) in self.data.into_iter().enumerate() {
      let data = data.into_rgba8();
      let (width, height) = data.dimensions();
      texture.write_texture_data(queue, data.as_raw(), 0, NonZeroU32::new(width * 4), None, Extent3d { width, height, depth_or_array_layers: idx as u32 });
    }
    let sampler = SamplerBuilder::new()
      .with_label(sampler_label)
      .build(device);
    let (sampler_layout_entry, sampler_bind_group_entry) = sampler.create_bind_group_entries(1, ShaderStage::FRAGMENT);
    let (bind_group_layout, bind_group) = CombinedBindGroupLayoutBuilder::new()
      .with_layout_entries(&[texture_layout_entry, sampler_layout_entry])
      .with_entries(&[texture_bind_group_entry, sampler_bind_group_entry])
      .with_layout_label(bind_group_layout_label)
      .with_label(bind_group_label)
      .build(device);
    Ok(ArrayTextureDef {
      texture,
      sampler,
      bind_group_layout,
      bind_group,
    })
  }
}

// Texture definition

pub struct ArrayTextureDef {
  pub texture: GfxTexture,
  pub sampler: Sampler,
  pub bind_group_layout: BindGroupLayout,
  pub bind_group: BindGroup,
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
