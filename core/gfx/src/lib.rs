use winit::window::Window;
use winit::event::WindowEvent;

struct State {
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  sc_desc: wgpu::SwapChainDescriptor,
  swap_chain: wgpu::SwapChain,
  size: winit::dpi::PhysicalSize<u32>,
}

impl State {
  // Following https://sotrh.github.io/learn-wgpu/beginner/tutorial2-swapchain/#state-new
  // Creating some of the wgpu types requires async code
  async fn new(window: &Window) -> Self {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(window) };
    let adapter = instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
      },
    ).await.unwrap();

    let (device, queue) = adapter.request_device(
      &wgpu::DeviceDescriptor {
        features: wgpu::Features::empty(),
        limits: wgpu::Limits::default(),
        label: None,
      },
      None, // Trace path
    ).await.unwrap();

    let sc_desc = wgpu::SwapChainDescriptor {
      usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
      format: adapter.get_swap_chain_preferred_format(&surface),
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
    };
    let swap_chain = device.create_swap_chain(&surface, &sc_desc);

    Self {
      surface,
      device,
      queue,
      sc_desc,
      swap_chain,
      size,
    }
  }

  fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    todo!()
  }

  fn input(&mut self, event: &WindowEvent) -> bool {
    todo!()
  }

  fn update(&mut self) {
    todo!()
  }

  fn render(&mut self) -> Result<(), wgpu::SwapChainError> {
    todo!()
  }
}
