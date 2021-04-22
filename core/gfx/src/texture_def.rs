use image::{DynamicImage, GenericImageView};
use thiserror::Error;
use wgpu::{Device, Texture, TextureFormat, Queue};

use common::idx_assigner;
use common::idx_assigner::IdxAssigner;

use crate::texture::TextureBuilder;
use std::num::NonZeroU32;

// Creation

pub struct TextureArrayDefBuilder {
  width: u32,
  height: u32,
  assigner: IdxAssigner<TextureIdx, u16>,
  data: Vec<DynamicImage>,
}

impl TextureArrayDefBuilder {
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

impl TextureArrayDefBuilder {
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

impl TextureArrayDefBuilder {
  pub fn build(self, device: &Device, queue: &Queue) -> Result<TextureDef, BuildTextureArrayError> {
    let layer_count = NonZeroU32::new(self.data.len() as u32).ok_or(BuildTextureArrayError::NoTexturesFail)?;
    let texture_array = TextureBuilder::new()
      .with_2d_array_size(self.width, self.height, layer_count)
      .with_rgba8_unorm_srgb_format()
      .build(device);
    texture_array.write_2d_rgba_image()
  }
}

// Texture definition

pub struct TextureDef {
  pub texture_array: Texture,
  pub descriptor_set_layout: DescriptorSetLayout,
  pub descriptor_pool: DescriptorPool,
  pub descriptor_set: DescriptorSet,
}

impl TextureDef {
  fn new(
    texture_array: Texture,
    descriptor_set_layout: DescriptorSetLayout,
    descriptor_pool: DescriptorPool,
    descriptor_set: DescriptorSet,
  ) -> Self {
    Self {
      texture_array,
      descriptor_set_layout,
      descriptor_pool,
      descriptor_set,
    }
  }

  // pub unsafe fn destroy(&self, device: &Device) {
  //   device.destroy_descriptor_pool(self.descriptor_pool);
  //   device.destroy_descriptor_set_layout(self.descriptor_set_layout);
  //   self.texture_array.destroy(device, allocator);
  // }
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
