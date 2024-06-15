use std::num::NonZeroU32;

use wgpu::{BindGroupLayoutEntry, BindingType, BufferBindingType, BufferSize, SamplerBindingType, ShaderStages, TextureSampleType, TextureViewDimension};

#[derive(Copy, Clone, Debug)]
pub struct BindGroupLayoutEntryBuilder<T = ()> {
  binding: u32,
  visibility: ShaderStages,
  ty: T,
  count: Option<NonZeroU32>,
}

impl Default for BindGroupLayoutEntryBuilder {
  #[inline]
  fn default() -> Self {
    Self {
      binding: 0,
      visibility: ShaderStages::empty(),
      ty: (),
      count: None,
    }
  }
}

impl BindGroupLayoutEntryBuilder {
  #[inline]
  pub fn new() -> Self { Self::default() }
}

impl<T> BindGroupLayoutEntryBuilder<T> {
  #[inline]
  pub fn binding_index(mut self, binding_index: u32) -> Self {
    self.binding = binding_index;
    self
  }

  #[inline]
  pub fn shader_visibility(mut self, visibility: ShaderStages) -> Self {
    self.visibility = visibility;
    self
  }
  /// [ShaderStages::VERTEX]
  #[inline]
  pub fn vertex_shader_visibility(self) -> Self {
    self.shader_visibility(ShaderStages::VERTEX)
  }
  /// [ShaderStages::FRAGMENT]
  #[inline]
  pub fn fragment_shader_visibility(self) -> Self {
    self.shader_visibility(ShaderStages::FRAGMENT)
  }
  /// [ShaderStages::COMPUTE]
  #[inline]
  pub fn compute_shader_visibility(self) -> Self {
    self.shader_visibility(ShaderStages::COMPUTE)
  }

  /// [BindingType::Buffer]
  #[inline]
  pub fn buffer(self) -> BindGroupLayoutEntryBuilder<BufferBindingBuilder> {
    self.replace_ty(BufferBindingBuilder::default())
  }
  /// [BindingType::Sampler]
  #[inline]
  pub fn sampler(self) -> BindGroupLayoutEntryBuilder<SamplerBindingBuilder> {
    self.replace_ty(SamplerBindingBuilder::default())
  }
  /// [BindingType::Texture]
  #[inline]
  pub fn texture(self) -> BindGroupLayoutEntryBuilder<TextureBindingBuilder> {
    self.replace_ty(TextureBindingBuilder::default())
  }

  #[inline]
  pub fn array_count(mut self, array_count: NonZeroU32) -> Self {
    self.count = Some(array_count);
    self
  }


  #[inline]
  fn replace_ty<TT>(self, ty: TT) -> BindGroupLayoutEntryBuilder<TT> {
    BindGroupLayoutEntryBuilder {
      binding: self.binding,
      visibility: self.visibility,
      ty,
      count: self.count,
    }
  }
}

impl<T: Into<BindingType>> BindGroupLayoutEntryBuilder<T> {
  #[inline]
  pub fn build(self) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
      binding: self.binding,
      visibility: self.visibility,
      ty: self.ty.into(),
      count: self.count,
    }
  }
}


/// Builder for [BindingType::Buffer].
#[derive(Default, Copy, Clone, Debug)]
pub struct BufferBindingBuilder {
  ty: BufferBindingType,
  has_dynamic_offset: bool,
  min_binding_size: Option<BufferSize>,
}

impl From<BufferBindingBuilder> for BindingType {
  fn from(builder: BufferBindingBuilder) -> Self {
    BindingType::Buffer {
      ty: builder.ty,
      has_dynamic_offset: builder.has_dynamic_offset,
      min_binding_size: builder.min_binding_size,
    }
  }
}

impl BindGroupLayoutEntryBuilder<BufferBindingBuilder> {
  #[inline]
  pub fn ty(mut self, ty: BufferBindingType) -> Self {
    self.ty.ty = ty;
    self
  }
  /// [BufferBindingType::Uniform]
  #[inline]
  pub fn uniform(self) -> Self {
    self.ty(BufferBindingType::Uniform)
  }
  /// [BufferBindingType::Storage]
  #[inline]
  pub fn storage(self, read_only: bool) -> Self {
    self.ty(BufferBindingType::Storage { read_only })
  }
  /// [BufferBindingType::Storage]
  #[inline]
  pub fn storage_read(self) -> Self {
    self.storage(true)
  }
  /// [BufferBindingType::Storage]
  #[inline]
  pub fn storage_read_write(self) -> Self {
    self.storage(false)
  }

  #[inline]
  pub fn dynamic_offset(mut self, has_dynamic_offset: bool) -> Self {
    self.ty.has_dynamic_offset = has_dynamic_offset;
    self
  }

  #[inline]
  pub fn min_binding_size(mut self, min_binding_size: Option<BufferSize>) -> Self {
    self.ty.min_binding_size = min_binding_size;
    self
  }
}


/// Builder for [BindingType::Sampler].
#[derive(Copy, Clone, Debug)]
pub struct SamplerBindingBuilder {
  ty: SamplerBindingType,
}

impl Default for SamplerBindingBuilder {
  #[inline]
  fn default() -> Self {
    Self { ty: SamplerBindingType::Filtering }
  }
}

impl From<SamplerBindingBuilder> for BindingType {
  fn from(builder: SamplerBindingBuilder) -> Self {
    BindingType::Sampler(builder.ty)
  }
}

impl BindGroupLayoutEntryBuilder<SamplerBindingBuilder> {
  #[inline]
  pub fn ty(mut self, ty: SamplerBindingType) -> Self {
    self.ty.ty = ty;
    self
  }
  /// [SamplerBindingType::Filtering]
  #[inline]
  pub fn filtering(self) -> Self {
    self.ty(SamplerBindingType::Filtering)
  }
  /// [SamplerBindingType::NonFiltering]
  #[inline]
  pub fn non_filtering(self) -> Self {
    self.ty(SamplerBindingType::NonFiltering)
  }
  /// [SamplerBindingType::Comparison]
  #[inline]
  pub fn comparison(self) -> Self {
    self.ty(SamplerBindingType::Comparison)
  }
}


/// Builder for [BindingType::Texture].
#[derive(Default, Copy, Clone, Debug)]
pub struct TextureBindingBuilder {
  sample_type: TextureSampleType,
  view_dimension: TextureViewDimension,
  multisampled: bool,
}

impl From<TextureBindingBuilder> for BindingType {
  fn from(builder: TextureBindingBuilder) -> Self {
    BindingType::Texture {
      sample_type: builder.sample_type,
      view_dimension: builder.view_dimension,
      multisampled: builder.multisampled,
    }
  }
}

impl BindGroupLayoutEntryBuilder<TextureBindingBuilder> {
  #[inline]
  pub fn sample_type(mut self, sample_type: TextureSampleType) -> Self {
    self.ty.sample_type = sample_type;
    self
  }
  /// [TextureSampleType::Float]
  #[inline]
  pub fn float(self, filterable: bool) -> Self {
    self.sample_type(TextureSampleType::Float { filterable })
  }
  /// [TextureSampleType::Float]
  #[inline]
  pub fn float_filterable(self) -> Self {
    self.float(true)
  }
  /// [TextureSampleType::Float]
  #[inline]
  pub fn float_non_filterable(self) -> Self {
    self.float(false)
  }
  /// [TextureSampleType::Depth]
  #[inline]
  pub fn depth(self) -> Self {
    self.sample_type(TextureSampleType::Depth)
  }
  /// [TextureSampleType::Sint]
  #[inline]
  pub fn signed_integer(self) -> Self {
    self.sample_type(TextureSampleType::Sint)
  }
  /// [TextureSampleType::Uint]
  #[inline]
  pub fn unsigned_integer(self) -> Self {
    self.sample_type(TextureSampleType::Uint)
  }

  #[inline]
  pub fn view_dimension(mut self, view_dimension: TextureViewDimension) -> Self {
    self.ty.view_dimension = view_dimension;
    self
  }
  /// [TextureViewDimension::D1]
  #[inline]
  pub fn d1(self) -> Self {
    self.view_dimension(TextureViewDimension::D1)
  }
  /// [TextureViewDimension::D2]
  #[inline]
  pub fn d2(self) -> Self {
    self.view_dimension(TextureViewDimension::D2)
  }
  /// [TextureViewDimension::D2Array]
  #[inline]
  pub fn d2_array(self) -> Self {
    self.view_dimension(TextureViewDimension::D2Array)
  }
  /// [TextureViewDimension::Cube]
  #[inline]
  pub fn cube(self) -> Self {
    self.view_dimension(TextureViewDimension::Cube)
  }
  /// [TextureViewDimension::CubeArray]
  #[inline]
  pub fn cube_array(self) -> Self {
    self.view_dimension(TextureViewDimension::CubeArray)
  }
  /// [TextureViewDimension::D3]
  #[inline]
  pub fn d3(self) -> Self {
    self.view_dimension(TextureViewDimension::D3)
  }

  #[inline]
  pub fn multisampled(mut self, multisampled: bool) -> Self {
    self.ty.multisampled = multisampled;
    self
  }
}
