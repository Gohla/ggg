use wgpu::{Adapter, Device, PresentMode, Surface, SurfaceConfiguration, TextureFormat};

use common::screen::ScreenSize;

#[derive(Debug)]
pub struct GfxSurface {
  inner: Surface,
  configuration: SurfaceConfiguration,
  size: ScreenSize,
}

impl GfxSurface {
  pub fn new(surface: Surface, adapter: &Adapter, device: &Device, present_mode: PresentMode, size: ScreenSize) -> GfxSurface {
    let configuration = Self::create_configuration(&surface, adapter, present_mode, size);
    surface.configure(device, &configuration);
    GfxSurface { inner: surface, configuration, size }
  }

  pub fn new_with_defaults(surface: Surface, adapter: &Adapter, device: &Device, size: ScreenSize) -> GfxSurface {
    Self::new(surface, adapter, device, wgpu::PresentMode::Mailbox, size)
  }


  pub fn get_texture_format(&self) -> TextureFormat {
    self.configuration.format
  }

  pub fn get_size(&self) -> ScreenSize { self.size }


  pub fn resize(&mut self, adapter: &Adapter, device: &Device, size: ScreenSize) {
    let configuration = Self::create_configuration(&self.inner, adapter, self.configuration.present_mode, size);
    self.inner.configure(device, &configuration);
    self.size = size;
  }


  fn create_configuration(surface: &Surface, adapter: &Adapter, present_mode: PresentMode, size: ScreenSize) -> SurfaceConfiguration {
    SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface.get_preferred_format(adapter)
        .unwrap_or_else(|| panic!("Surface is incompatible with the adapter")),
      width: size.physical.width as u32,
      height: size.physical.height as u32,
      present_mode,
    }
  }
}

impl GfxSurface {
  #[inline]
  pub fn get_inner(&self) -> &Surface { &self.inner }
}

impl std::ops::Deref for GfxSurface {
  type Target = Surface;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.get_inner() }
}
