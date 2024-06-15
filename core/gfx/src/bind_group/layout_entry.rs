use std::num::NonZeroU32;

use wgpu::{BindGroupLayoutEntry, BindingType, BufferBindingType, BufferSize, Features, SamplerBindingType, ShaderStages, TextureAspect, TextureFormat, TextureSampleType, TextureViewDimension};

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
  pub fn binding(mut self, binding: u32) -> Self {
    self.binding = binding;
    self
  }

  #[inline]
  pub fn visibility(mut self, visibility: ShaderStages) -> Self {
    self.visibility = visibility;
    self
  }
  /// [ShaderStages::NONE]
  #[inline]
  pub fn no_visibility(self) -> Self {
    self.visibility(ShaderStages::NONE)
  }
  /// [ShaderStages::VERTEX]
  #[inline]
  pub fn vertex_visibility(self) -> Self {
    self.visibility(ShaderStages::VERTEX)
  }
  /// [ShaderStages::FRAGMENT]
  #[inline]
  pub fn fragment_visibility(self) -> Self {
    self.visibility(ShaderStages::FRAGMENT)
  }
  /// [ShaderStages::VERTEX_FRAGMENT]
  #[inline]
  pub fn vertex_fragment_visibility(self) -> Self {
    self.visibility(ShaderStages::VERTEX_FRAGMENT)
  }
  /// [ShaderStages::COMPUTE]
  #[inline]
  pub fn compute_visibility(self) -> Self {
    self.visibility(ShaderStages::COMPUTE)
  }

  /// [BindingType::Buffer]
  #[inline]
  pub fn buffer(self) -> BindGroupLayoutEntryBuilder<BufferLayoutBuilder> {
    self.replace_ty(BufferLayoutBuilder::default())
  }
  /// [BindingType::Sampler]
  #[inline]
  pub fn sampler(self) -> BindGroupLayoutEntryBuilder<SamplerLayoutBuilder> {
    self.replace_ty(SamplerLayoutBuilder::default())
  }
  /// [BindingType::Texture]
  #[inline]
  pub fn texture(self) -> BindGroupLayoutEntryBuilder<TextureLayoutBuilder> {
    self.replace_ty(TextureLayoutBuilder::default())
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
pub struct BufferLayoutBuilder {
  ty: BufferBindingType,
  has_dynamic_offset: bool,
  min_binding_size: Option<BufferSize>,
}
impl From<BufferLayoutBuilder> for BindingType {
  #[inline]
  fn from(builder: BufferLayoutBuilder) -> Self {
    BindingType::Buffer {
      ty: builder.ty,
      has_dynamic_offset: builder.has_dynamic_offset,
      min_binding_size: builder.min_binding_size,
    }
  }
}
impl BindGroupLayoutEntryBuilder<BufferLayoutBuilder> {
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
pub struct SamplerLayoutBuilder {
  ty: SamplerBindingType,
}
impl Default for SamplerLayoutBuilder {
  #[inline]
  fn default() -> Self { Self { ty: SamplerBindingType::Filtering } }
}
impl From<SamplerLayoutBuilder> for BindingType {
  #[inline]
  fn from(builder: SamplerLayoutBuilder) -> Self { BindingType::Sampler(builder.ty) }
}
impl BindGroupLayoutEntryBuilder<SamplerLayoutBuilder> {
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
pub struct TextureLayoutBuilder {
  sample_type: TextureSampleType,
  format: Option<TextureFormat>,
  aspect: TextureAspect,
  device_features: Features,
  view_dimension: TextureViewDimension,
  multisampled: bool,
}
impl From<TextureLayoutBuilder> for BindingType {
  #[inline]
  fn from(builder: TextureLayoutBuilder) -> Self {
    let sample_type = builder.format
      .and_then(|format| format.sample_type(Some(builder.aspect), Some(builder.device_features)))
      .unwrap_or(builder.sample_type);
    BindingType::Texture {
      sample_type,
      view_dimension: builder.view_dimension,
      multisampled: builder.multisampled,
    }
  }
}
impl BindGroupLayoutEntryBuilder<TextureLayoutBuilder> {
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

  /// Set the format of textures that will be used for this binding layout. When set, the
  /// [sample_type](Self::sample_type) will be determined by [TextureFormat::sample_type] if it returns `Some`. Set
  /// [aspect](Self::aspect) and [device_features](Self::device_features) for a more correct sample type.
  #[inline]
  pub fn format(mut self, format: TextureFormat) -> Self {
    self.ty.format = Some(format);
    self
  }
  #[inline]
  pub fn aspect(mut self, aspect: TextureAspect) -> Self {
    self.ty.aspect = aspect;
    self
  }
  #[inline]
  pub fn device_features(mut self, device_features: Features) -> Self {
    self.ty.device_features = device_features;
    self
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
