use wgpu::{Adapter, BlendState, ColorTargetState, CompositeAlphaMode, Device, PresentMode, Surface, SurfaceConfiguration, TextureFormat};

use common::screen::ScreenSize;

#[derive(Debug)]
pub struct GfxSurface {
  inner: Surface<'static>,
  configuration: SurfaceConfiguration,
  viewport: ScreenSize,

  pub non_blend_target: [Option<ColorTargetState>; 1],
  pub replace_blend_target: [Option<ColorTargetState>; 1],
  pub alpha_blend_target: [Option<ColorTargetState>; 1],
  pub premultiplied_alpha_blend_target: [Option<ColorTargetState>; 1],
}

impl GfxSurface {
  pub fn new(surface: Surface<'static>, adapter: &Adapter, device: &Device, present_mode: PresentMode, viewport: ScreenSize) -> Self {
    let configuration = Self::create_configuration(&surface, adapter, present_mode, viewport);
    surface.configure(device, &configuration);

    let non_blend_target = [Some(configuration.format.into())];
    let replace_blend_target = [Some(ColorTargetState {
      blend: Some(BlendState::REPLACE),
      ..configuration.format.into()
    })];
    let alpha_blend_target = [Some(ColorTargetState {
      blend: Some(BlendState::ALPHA_BLENDING),
      ..configuration.format.into()
    })];
    let premultiplied_alpha_blend_target = [Some(ColorTargetState {
      blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
      ..configuration.format.into()
    })];

    Self { inner: surface, configuration, viewport, non_blend_target, replace_blend_target, alpha_blend_target, premultiplied_alpha_blend_target }
  }
  pub fn new_with_defaults(surface: Surface<'static>, adapter: &Adapter, device: &Device, size: ScreenSize) -> Self {
    Self::new(surface, adapter, device, PresentMode::Mailbox, size)
  }

  #[inline]
  pub fn get_configuration(&self) -> &SurfaceConfiguration { &self.configuration }
  #[inline]
  pub fn get_swapchain_texture_format(&self) -> TextureFormat { self.configuration.format }
  #[inline]
  pub fn get_viewport(&self) -> ScreenSize { self.viewport }


  pub fn resize(&mut self, adapter: &Adapter, device: &Device, viewport: ScreenSize) {
    let configuration = Self::create_configuration(&self.inner, adapter, self.configuration.present_mode, viewport);
    self.inner.configure(device, &configuration);
    self.viewport = viewport;
  }


  fn create_configuration(surface: &Surface, adapter: &Adapter, present_mode: PresentMode, viewport: ScreenSize) -> SurfaceConfiguration {
    let capabilities = surface.get_capabilities(adapter);
    tracing::debug!(?capabilities, "Queried surface capabilities");
    SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: *capabilities.formats.get(0)
        .unwrap_or_else(|| panic!("No supported formats; surface is incompatible with the adapter")),
      width: viewport.physical.width as u32,
      height: viewport.physical.height as u32,
      present_mode, // TODO: check against capabilities?
      desired_maximum_frame_latency: 1, // TODO: make configurable
      alpha_mode: CompositeAlphaMode::Auto, // TODO: make configurable
      view_formats: vec![],
    }
  }
}

impl GfxSurface {
  #[inline]
  pub fn get_inner(&self) -> &Surface<'static> { &self.inner }
}

impl std::ops::Deref for GfxSurface {
  type Target = Surface<'static>;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.get_inner() }
}
