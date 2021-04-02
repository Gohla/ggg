use std::sync::mpsc::Receiver;
use std::thread;

use dotenv;
use thiserror::Error;
use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt};
use tracing_subscriber::prelude::*;
use wgpu::SwapChainError;

use gfx::{AdapterRequestError, DeviceRequestError, GfxAdapter, GfxDevice, GfxInstance, GfxSurface, GfxSwapChain};
use math::prelude::*;
use os::context::OsContext;
use os::event_sys::{OsEvent, OsEventSys};
use os::input_sys::OsInputSys;
use os::window::{Window, WindowCreateError};
use util::timing::{Duration, FrameTime, FrameTimer, TickTimer};

#[derive(Error, Debug)]
pub enum CreateError {
  #[error(transparent)]
  WindowCreateFail(#[from] WindowCreateError),
  #[error(transparent)]
  AdapterRequestFail(#[from] AdapterRequestError),
  #[error(transparent)]
  RequestDeviceFail(#[from] DeviceRequestError),
  #[error(transparent)]
  ThreadCreateFail(#[from] std::io::Error),
}

pub async fn run() -> Result<(), CreateError> {
  dotenv::dotenv().ok();

  let fmt_layer = fmt::layer()
    .with_writer(std::io::stderr)
    ;
  let filter_layer = EnvFilter::from_default_env();
  tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .init();

  let os_context = OsContext::new();
  let window = {
    let window_min_size = LogicalSize::new(1920.0, 1080.0);
    Window::new(&os_context, window_min_size, window_min_size, "SG")?
  };
  let (os_event_sys, os_event_rx, os_input_sys) = {
    let (event_sys, input_event_rx, event_rx) = OsEventSys::new(&window);
    let input_sys = OsInputSys::new(input_event_rx);
    (event_sys, event_rx, input_sys)
  };

  let instance = GfxInstance::new_with_primary_backends();
  let surface = unsafe { instance.create_surface(window.get_inner()) };
  let adapter = instance.request_low_power_adapter(&surface).await?;
  let device = adapter.request_device(&wgpu::DeviceDescriptor {
    ..wgpu::DeviceDescriptor::default()
  }, None).await?;
  let swap_chain = device.create_swap_chain_with_defaults(&surface, &adapter, window.get_inner_size());

  thread::Builder::new()
    .name("Application".to_string())
    .spawn(move || {
      debug!("Application thread started");
      run_loop(window, os_event_rx, os_input_sys, instance, surface, adapter, device, swap_chain);
      debug!("Application thread stopped");
    })?;

  debug!("Main thread OS-event loop started");
  os_event_sys.run(os_context); // Hijacks the current thread. All code after the this line is ignored!
  Ok(()) // Ignored, but needed to conform to the return type.
}

fn run_loop(
  window: Window,
  os_event_rx: Receiver<OsEvent>,
  mut os_input_sys: OsInputSys,
  _instance: GfxInstance,
  surface: GfxSurface,
  adapter: GfxAdapter,
  device: GfxDevice,
  mut swap_chain: GfxSwapChain,
) {
  let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../../target/shader/triangle.vert.spv"));
  let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../../target/shader/triangle.frag.spv"));
  let render_pipeline_layout =
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[],
      push_constant_ranges: &[],
    });
  let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Render Pipeline"),
    layout: Some(&render_pipeline_layout),
    vertex: wgpu::VertexState {
      module: &vs_module,
      entry_point: "main",
      buffers: &[],
    },
    fragment: Some(wgpu::FragmentState {
      module: &fs_module,
      entry_point: "main",
      targets: &[wgpu::ColorTargetState {
        format: swap_chain.get_format(), // TODO: need to recreate if swap chain changes!
        alpha_blend: wgpu::BlendState::REPLACE,
        color_blend: wgpu::BlendState::REPLACE,
        write_mask: wgpu::ColorWrite::ALL,
      }],
    }),
    primitive: wgpu::PrimitiveState {
      topology: wgpu::PrimitiveTopology::TriangleList,
      strip_index_format: None,
      front_face: wgpu::FrontFace::Ccw,
      cull_mode: wgpu::CullMode::Back,
      // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
      polygon_mode: wgpu::PolygonMode::Fill,
    },
    depth_stencil: None,
    multisample: wgpu::MultisampleState {
      count: 1,
      mask: !0,
      alpha_to_coverage_enabled: false,
    },
  });


  let mut frame_timer = FrameTimer::new();
  let mut tick_timer = TickTimer::new(Duration::from_ns(16_666_667));
  let mut recreate_swap_chain = false;

  'main: loop {
    if recreate_swap_chain {
      swap_chain = swap_chain.resize(&surface, &adapter, &device, window.get_inner_size());
      recreate_swap_chain = false;
    }

    // Timing
    let FrameTime { frame_time, .. } = frame_timer.frame();
    tick_timer.update_lag(frame_time);
    // Process OS events
    for os_event in os_event_rx.try_iter() {
      match os_event {
        OsEvent::TerminateRequested => break 'main,
        _ => {}
      }
    }
    // Process input
    let _raw_input = os_input_sys.update();
    // Simulate tick
    if tick_timer.should_tick() {
      while tick_timer.should_tick() { // Run simulation.
        tick_timer.tick_start();
        // TODO: run simulation
        tick_timer.tick_end();
      }
    }

    // Get frame to draw into
    let frame = match swap_chain.get_current_frame() {
      Ok(frame) => frame,
      Err(e) => {
        match e {
          SwapChainError::Outdated => recreate_swap_chain = true,
          SwapChainError::Lost => recreate_swap_chain = true,
          SwapChainError::OutOfMemory => panic!("Allocating swap chain frame reported out of memory; stopping"),
          _ => {}
        };
        continue;
      }
    };
    if frame.suboptimal {
      recreate_swap_chain = true;
    }

    // Create command encoder
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });

    { // Setup render pass
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[
          wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &frame.output.view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
              }),
              store: true,
            },
          }
        ],
        depth_stencil_attachment: None,
      });
      render_pass.set_pipeline(&render_pipeline);
      render_pass.draw(0..3, 0..1);
    }

    // Finish command encoder to retrieve a command buffer.
    let command_buffer = encoder.finish();
    // Submit command encoder and draw.
    device.inner_queue().submit(std::iter::once(command_buffer));
  }
}
