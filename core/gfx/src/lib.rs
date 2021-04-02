use std::ops::Deref;

use thiserror::Error;

use math::screen::ScreenSize;

pub struct GfxInstance {
  inner: wgpu::Instance,
}

pub struct GfxSurface {
  inner: wgpu::Surface,
}

pub struct GfxAdapter {
  inner: wgpu::Adapter,
}

pub struct GfxDevice {
  inner: wgpu::Device,
  queue: wgpu::Queue,
}

pub struct GfxSwapChain {
  inner: wgpu::SwapChain,
  descriptor: wgpu::SwapChainDescriptor,
}


// Instance creation.

impl GfxInstance {
  pub fn new(backends: wgpu::BackendBit) -> Self {
    let instance = wgpu::Instance::new(backends);
    Self { inner: instance }
  }

  pub fn new_with_primary_backends() -> Self {
    Self::new(wgpu::BackendBit::PRIMARY)
  }
}


// Surface creation

impl GfxInstance {
  pub unsafe fn create_surface(&self, window: &impl raw_window_handle::HasRawWindowHandle) -> GfxSurface {
    GfxSurface::new(self.inner.create_surface(window))
  }
}

impl GfxSurface {
  fn new(surface: wgpu::Surface) -> Self {
    Self { inner: surface }
  }
}


// Adapter creation

#[derive(Error, Debug)]
#[error("Failed to request graphics adapter because no adapters were found that meet the required options")]
pub struct AdapterRequestError;

impl GfxInstance {
  pub async fn request_adapter(&self, options: &wgpu::RequestAdapterOptions<'_>) -> Result<GfxAdapter, AdapterRequestError> {
    self.inner.request_adapter(&options).await.map(|a| GfxAdapter::new(a)).ok_or(AdapterRequestError)
  }

  pub async fn request_low_power_adapter(&self, surface: &GfxSurface) -> Result<GfxAdapter, AdapterRequestError> {
    self.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::LowPower,
      compatible_surface: Some(&surface),
      ..wgpu::RequestAdapterOptions::default()
    }).await
  }
}

impl GfxAdapter {
  pub fn new(adapter: wgpu::Adapter) -> Self {
    Self { inner: adapter }
  }
}


// Device creation

#[derive(Error, Debug)]
#[error("Failed to request graphics device because no adapters were found that meet the required options")]
pub struct DeviceRequestError(#[from] wgpu::RequestDeviceError);

impl GfxAdapter {
  pub async fn request_device(&self, descriptor: &wgpu::DeviceDescriptor<'_>, trace_path: Option<&std::path::Path>) -> Result<GfxDevice, DeviceRequestError> {
    let (device, queue) = self.inner.request_device(descriptor, trace_path).await?;
    Ok(GfxDevice::new(device, queue))
  }
}

impl GfxDevice {
  pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
    Self { inner: device, queue }
  }
}


// Swap chain creation, resize, and utility

impl GfxDevice {
  pub fn create_swap_chain(&self, surface: &GfxSurface, adapter: &GfxAdapter, present_mode: wgpu::PresentMode, size: ScreenSize) -> GfxSwapChain {
    let descriptor = wgpu::SwapChainDescriptor {
      usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
      format: adapter.get_swap_chain_preferred_format(&surface),
      width: size.physical.width,
      height: size.physical.height,
      present_mode,
    };
    GfxSwapChain::new(self.inner.create_swap_chain(&surface, &descriptor), descriptor)
  }

  pub fn create_swap_chain_with_defaults(&self, surface: &GfxSurface, adapter: &GfxAdapter, size: ScreenSize) -> GfxSwapChain {
    self.create_swap_chain(surface, adapter, wgpu::PresentMode::Mailbox, size)
  }
}

impl GfxSwapChain {
  pub fn new(swap_chain: wgpu::SwapChain, descriptor: wgpu::SwapChainDescriptor) -> Self {
    Self { inner: swap_chain, descriptor }
  }

  pub fn resize(self, surface: &GfxSurface, adapter: &GfxAdapter, device: &GfxDevice, size: ScreenSize) -> GfxSwapChain {
    device.create_swap_chain(surface, adapter, self.descriptor.present_mode, size)
  }

  pub fn get_format(&self) -> wgpu::TextureFormat {
    self.descriptor.format
  }
}


// Getters and Deref implementations

// Instance

impl GfxInstance {
  #[inline]
  pub fn inner(&self) -> &wgpu::Instance { &self.inner }
}

impl Deref for GfxInstance {
  type Target = wgpu::Instance;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.inner() }
}

// Surface

impl GfxSurface {
  #[inline]
  pub fn inner(&self) -> &wgpu::Surface { &self.inner }
}

impl Deref for GfxSurface {
  type Target = wgpu::Surface;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.inner() }
}

// Adapter

impl GfxAdapter {
  #[inline]
  pub fn inner(&self) -> &wgpu::Adapter { &self.inner }
}

impl Deref for GfxAdapter {
  type Target = wgpu::Adapter;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.inner() }
}

// Device

impl GfxDevice {
  #[inline]
  pub fn inner(&self) -> &wgpu::Device { &self.inner }
  #[inline]
  pub fn inner_queue(&self) -> &wgpu::Queue { &self.queue }
}

impl Deref for GfxDevice {
  type Target = wgpu::Device;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.inner() }
}

// Swap chain

impl GfxSwapChain {
  #[inline]
  pub fn inner(&self) -> &wgpu::SwapChain { &self.inner }
}

impl Deref for GfxSwapChain {
  type Target = wgpu::SwapChain;
  #[inline]
  fn deref(&self) -> &Self::Target { &self.inner() }
}
