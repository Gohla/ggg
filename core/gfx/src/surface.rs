use wgpu::{Adapter, CompositeAlphaMode, Device, PresentMode, Surface, SurfaceConfiguration, TextureFormat};

use common::screen::ScreenSize;

#[derive(Debug)]
pub struct GfxSurface<'w> {
  inner: Surface<'w>,
  configuration: SurfaceConfiguration,
  size: ScreenSize,
}

impl<'w> GfxSurface<'w> {
  pub fn new(surface: Surface<'w>, adapter: &Adapter, device: &Device, present_mode: PresentMode, size: ScreenSize) -> Self {
    let configuration = Self::create_configuration(&surface, adapter, present_mode, size);
    surface.configure(device, &configuration);
    Self { inner: surface, configuration, size }
  }
  pub fn new_with_defaults(surface: Surface<'w>, adapter: &Adapter, device: &Device, size: ScreenSize) -> Self {
    Self::new(surface, adapter, device, PresentMode::Mailbox, size)
  }


  pub fn get_configuration(&self) -> &SurfaceConfiguration { &self.configuration }
  pub fn get_format(&self) -> TextureFormat { self.configuration.format }
  pub fn get_size(&self) -> ScreenSize { self.size }


  pub fn resize(&mut self, adapter: &Adapter, device: &Device, size: ScreenSize) {
    let configuration = Self::create_configuration(&self.inner, adapter, self.configuration.present_mode, size);
    self.inner.configure(device, &configuration);
    self.size = size;
  }


  fn create_configuration(surface: &Surface, adapter: &Adapter, present_mode: PresentMode, size: ScreenSize) -> SurfaceConfiguration {
    let capabilities = surface.get_capabilities(adapter);
    tracing::debug!(?capabilities, "Queried surface capabilities");
    SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: *capabilities.formats.get(0)
        .unwrap_or_else(|| panic!("No supported formats; surface is incompatible with the adapter")),
      width: size.physical.width as u32,
      height: size.physical.height as u32,
      present_mode, // TODO: check against capabilities?
      desired_maximum_frame_latency: 1, // TODO: make configurable
      alpha_mode: CompositeAlphaMode::Auto, // TODO: make configurable
      view_formats: vec![],
    }
  }
}

impl<'w> GfxSurface<'w> {
  #[inline]
  pub fn get_inner(&self) -> &Surface<'w> { &self.inner }
}

impl<'w> std::ops::Deref for GfxSurface<'w> {
  type Target = Surface<'w>;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.get_inner() }
}
