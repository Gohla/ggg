use wgpu::{Adapter, Device, PresentMode, Surface, SwapChain, SwapChainDescriptor, TextureFormat};

use common::screen::ScreenSize;

pub struct GfxSwapChain {
  inner: SwapChain,
  descriptor: SwapChainDescriptor,
}

impl GfxSwapChain {
  pub fn new(surface: &Surface, adapter: &Adapter, device: &Device, present_mode: PresentMode, size: ScreenSize) -> GfxSwapChain {
    let descriptor = SwapChainDescriptor {
      usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
      format: adapter.get_swap_chain_preferred_format(&surface),
      width: size.physical.width,
      height: size.physical.height,
      present_mode,
    };
    let swap_chain = device.create_swap_chain(&surface, &descriptor);
    GfxSwapChain { inner: swap_chain, descriptor }
  }

  pub fn new_with_defaults(surface: &Surface, adapter: &Adapter, device: &Device, size: ScreenSize) -> GfxSwapChain {
    Self::new(surface, adapter, device, wgpu::PresentMode::Mailbox, size)
  }


  pub fn get_texture_format(&self) -> TextureFormat {
    self.descriptor.format
  }

  pub fn get_size(&self) -> (u32, u32) { (self.descriptor.width, self.descriptor.height) }


  pub fn resize(self, surface: &Surface, adapter: &Adapter, device: &Device, size: ScreenSize) -> GfxSwapChain {
    GfxSwapChain::new(surface, adapter, device, self.descriptor.present_mode, size)
  }
}

impl GfxSwapChain {
  #[inline]
  pub fn get_inner(&self) -> &SwapChain { &self.inner }
}

impl std::ops::Deref for GfxSwapChain {
  type Target = SwapChain;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.get_inner() }
}
