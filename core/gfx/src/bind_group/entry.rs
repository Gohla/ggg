use wgpu::{BindGroupEntry, BindingResource, Buffer, BufferAddress, BufferBinding, BufferSize, Sampler, TextureView};

pub struct BindGroupEntryBuilder<R = ()> {
  binding: u32,
  resource: R,
}

impl Default for BindGroupEntryBuilder {
  fn default() -> Self {
    Self { binding: 0, resource: () }
  }
}
impl BindGroupEntryBuilder {
  #[inline]
  pub fn new() -> Self { Self::default() }
}

impl<R> BindGroupEntryBuilder<R> {
  #[inline]
  pub fn binding(mut self, binding: u32) -> Self {
    self.binding = binding;
    self
  }

  /// [BindingResource::Buffer]
  #[inline]
  pub fn buffer(self, buffer: &Buffer) -> BindGroupEntryBuilder<BufferEntryBuilder> {
    self.replace_resource(BufferEntryBuilder::new(buffer))
  }
  /// [BindingResource::BufferArray]
  #[inline]
  pub fn buffer_array<'a>(self, buffer_binding_array: &'a [BufferBinding<'a>]) -> BindGroupEntryBuilder<BufferArrayEntryBuilder<'a>> {
    self.replace_resource(BufferArrayEntryBuilder::new(buffer_binding_array))
  }
  /// [BindingResource::Sampler]
  #[inline]
  pub fn sampler(self, sampler: &Sampler) -> BindGroupEntryBuilder<SamplerEntryBuilder> {
    self.replace_resource(SamplerEntryBuilder::new(sampler))
  }
  /// [BindingResource::SamplerArray]
  #[inline]
  pub fn sampler_array<'a>(self, sampler_array: &'a [&'a Sampler]) -> BindGroupEntryBuilder<SamplerArrayEntryBuilder<'a>> {
    self.replace_resource(SamplerArrayEntryBuilder::new(sampler_array))
  }
  /// [BindingResource::TextureView]
  #[inline]
  pub fn texture_view(self, texture_view: &TextureView) -> BindGroupEntryBuilder<TextureViewEntryBuilder> {
    self.replace_resource(TextureViewEntryBuilder::new(texture_view))
  }
  /// [BindingResource::TextureViewArray]
  #[inline]
  pub fn texture_view_array<'a>(self, texture_view_array: &'a [&'a TextureView]) -> BindGroupEntryBuilder<TextureViewArrayEntryBuilder<'a>> {
    self.replace_resource(TextureViewArrayEntryBuilder::new(texture_view_array))
  }

  #[inline]
  fn replace_resource<RR>(self, resource: RR) -> BindGroupEntryBuilder<RR> {
    BindGroupEntryBuilder { binding: self.binding, resource }
  }
}

impl<'a, R: Into<BindingResource<'a>>> BindGroupEntryBuilder<R> {
  #[inline]
  pub fn build(self) -> BindGroupEntry<'a> {
    BindGroupEntry {
      binding: self.binding,
      resource: self.resource.into(),
    }
  }
}


/// Builder for [BindingResource::Buffer].
#[derive(Copy, Clone, Debug)]
pub struct BufferEntryBuilder<'a> {
  buffer: &'a Buffer,
  offset: BufferAddress,
  size: Option<BufferSize>,
}
impl<'a> BufferEntryBuilder<'a> {
  #[inline]
  pub fn new(buffer: &'a Buffer) -> Self { Self { buffer, offset: 0, size: None } }
}
impl<'a> From<BufferEntryBuilder<'a>> for BindingResource<'a> {
  #[inline]
  fn from(builder: BufferEntryBuilder<'a>) -> Self {
    BindingResource::Buffer(BufferBinding {
      buffer: builder.buffer,
      offset: builder.offset,
      size: builder.size,
    })
  }
}
impl BindGroupEntryBuilder<BufferEntryBuilder<'_>> {
  #[inline]
  pub fn offset(mut self, offset: BufferAddress) -> Self {
    self.resource.offset = offset;
    self
  }
  #[inline]
  pub fn size(mut self, size: BufferSize) -> Self {
    self.resource.size = Some(size);
    self
  }
  #[inline]
  pub fn whole_buffer(mut self) -> Self {
    self.resource.size = None;
    self
  }
}


/// Builder for [BindingResource::BufferArray].
#[derive(Copy, Clone, Debug)]
pub struct BufferArrayEntryBuilder<'a> {
  buffer_binding_array: &'a [BufferBinding<'a>],
}
impl<'a> BufferArrayEntryBuilder<'a> {
  #[inline]
  pub fn new(buffer_binding_array: &'a [BufferBinding<'a>]) -> Self { Self { buffer_binding_array } }
}
impl<'a> From<BufferArrayEntryBuilder<'a>> for BindingResource<'a> {
  #[inline]
  fn from(builder: BufferArrayEntryBuilder<'a>) -> Self {
    BindingResource::BufferArray(builder.buffer_binding_array)
  }
}


/// Builder for [BindingResource::Sampler].
#[derive(Copy, Clone, Debug)]
pub struct SamplerEntryBuilder<'a> {
  sampler: &'a Sampler,
}
impl<'a> SamplerEntryBuilder<'a> {
  #[inline]
  pub fn new(sampler: &'a Sampler) -> Self { Self { sampler } }
}
impl<'a> From<SamplerEntryBuilder<'a>> for BindingResource<'a> {
  #[inline]
  fn from(builder: SamplerEntryBuilder<'a>) -> Self {
    BindingResource::Sampler(builder.sampler)
  }
}


/// Builder for [BindingResource::SamplerArray].
#[derive(Copy, Clone, Debug)]
pub struct SamplerArrayEntryBuilder<'a> {
  sampler_array: &'a [&'a Sampler],
}
impl<'a> SamplerArrayEntryBuilder<'a> {
  #[inline]
  pub fn new(sampler_array: &'a [&'a Sampler]) -> Self { Self { sampler_array } }
}
impl<'a> From<SamplerArrayEntryBuilder<'a>> for BindingResource<'a> {
  #[inline]
  fn from(builder: SamplerArrayEntryBuilder<'a>) -> Self {
    BindingResource::SamplerArray(builder.sampler_array)
  }
}


/// Builder for [BindingResource::TextureView].
#[derive(Copy, Clone, Debug)]
pub struct TextureViewEntryBuilder<'a> {
  texture_view: &'a TextureView,
}
impl<'a> TextureViewEntryBuilder<'a> {
  #[inline]
  pub fn new(texture_view: &'a TextureView) -> Self { Self { texture_view } }
}
impl<'a> From<TextureViewEntryBuilder<'a>> for BindingResource<'a> {
  #[inline]
  fn from(builder: TextureViewEntryBuilder<'a>) -> Self {
    BindingResource::TextureView(builder.texture_view)
  }
}


/// Builder for [BindingResource::TextureViewArray].
#[derive(Copy, Clone, Debug)]
pub struct TextureViewArrayEntryBuilder<'a> {
  texture_view_array: &'a [&'a TextureView],
}
impl<'a> TextureViewArrayEntryBuilder<'a> {
  #[inline]
  pub fn new(texture_view_array: &'a [&'a TextureView]) -> Self { Self { texture_view_array } }
}
impl<'a> From<TextureViewArrayEntryBuilder<'a>> for BindingResource<'a> {
  #[inline]
  fn from(builder: TextureViewArrayEntryBuilder<'a>) -> Self {
    BindingResource::TextureViewArray(builder.texture_view_array)
  }
}
