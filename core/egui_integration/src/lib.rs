use egui::{ClippedPrimitive, Context, epaint, FullOutput, PlatformOutput, Pos2, Rect, TexturesDelta};
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

use ::os::window::Window;
use common::input::RawInput;
use common::screen::ScreenSize;

use crate::gfx::GuiGfx;
use crate::os::GuiOs;

mod gfx;
mod os;

pub struct GuiIntegration {
  context: Context,
  os: GuiOs,
  gfx: GuiGfx,
}

// Creation

impl GuiIntegration {
  pub fn new(
    memory: Option<egui::Memory>,
    device: &Device,
    swap_chain_texture_format: TextureFormat,
  ) -> Self {
    let context = Context::default();
    if let Some(memory) = memory {
      context.memory_mut(|m| *m = memory);
    }
    let os = GuiOs::new();
    let gfx = GuiGfx::new(device, swap_chain_texture_format);
    Self { context, os, gfx }
  }

  #[inline]
  pub fn into_context(self) -> Context {
    self.context
  }


  /// Process `input`. Only processes keyboard input if `process_keyboard` is `true`. Only processes mouse input if
  /// `process_mouse` is `true`.
  #[inline]
  pub fn process_input(&mut self, input: &RawInput, process_keyboard: bool, process_mouse: bool) {
    self.os.process_input(input, process_keyboard, process_mouse);
  }

  /// Process a "cursor entered/left window" event.
  #[inline]
  pub fn process_window_cursor_event(&mut self, cursor_in_window: bool) {
    self.os.process_window_cursor_event(cursor_in_window);
  }

  /// Process a "window focus gained/lost" event.
  #[inline]
  pub fn process_window_focus_event(&mut self, focus: bool) {
    self.os.process_window_focus_event(focus);
  }


  /// Begin a new GUI frame, returning the context to start building the GUI.
  #[profiling::function]
  pub fn begin_frame(
    &mut self,
    viewport: ScreenSize,
    elapsed_time_in_seconds: f64,
    predicted_duration_in_seconds: f32,
  ) -> Context {
    let mut input = self.os.input();

    let screen_rect = Rect::from_min_size(Pos2::ZERO, viewport.physical.into());
    input.screen_rect = Some(screen_rect);

    let native_pixels_per_point: f64 = viewport.scale.into();
    if let Some(viewport) = input.viewports.get_mut(&input.viewport_id) {
      viewport.native_pixels_per_point = Some(native_pixels_per_point as f32);
    }

    input.time = Some(elapsed_time_in_seconds);
    input.predicted_dt = predicted_duration_in_seconds;

    self.context.begin_frame(input);
    self.context.clone()
  }


  /// End the current GUI frame, returning the full output for handling platform events and rendering.
  #[profiling::function]
  pub fn end_frame(&mut self) -> FullOutput {
    self.context.end_frame()
  }

  /// Process `platform_output` from [FullOutput] returned in [end_frame](Self::end_frame):
  ///
  /// - Set the cursor icon.
  /// - open URL in browser if a hyperlink was clicked.
  /// - Copy text to the clipboard if text was copied.
  #[inline]
  pub fn process_platform_output(&mut self, window: &Window, platform_output: PlatformOutput) {
    self.os.process_platform_output(window, platform_output);
  }

  /// Upload texture changes in `textures_delta` to the GPU. Get [TexturesDelta] from [FullOutput] returned by
  /// [end_frame](Self::end_frame). Call *before* [render](Self::render).
  #[inline]
  pub fn update_textures(&mut self, device: &Device, queue: &Queue, textures_delta: TexturesDelta) {
    self.gfx.update_textures(device, queue, textures_delta);
  }

  /// Tessellate `shapes` at `pixels_per_point`. Get [`Vec<epaint::ClippedShape>`] and `pixels_per_point` from
  /// [FullOutput] returned by [end_frame](Self::end_frame).
  #[inline]
  #[profiling::function]
  pub fn tessellate(&mut self, shapes: Vec<epaint::ClippedShape>, pixels_per_point: f32) -> Vec<ClippedPrimitive> {
    self.context.tessellate(shapes, pixels_per_point)
  }

  /// Render `clipped_primitives` onto `surface_texture_view` with `encoder`.
  #[inline]
  pub fn render(
    &mut self,
    device: &Device,
    queue: &Queue,
    clipped_primitives: Vec<ClippedPrimitive>,
    viewport: ScreenSize,
    output_texture: &TextureView,
    encoder: &mut CommandEncoder,
  ) {
    self.gfx.render(device, queue, clipped_primitives, viewport, output_texture, encoder);
  }


  /// End the current GUI frame and handle the output internally.
  #[profiling::function]
  pub fn end_frame_and_handle(
    &mut self,
    window: &Window,
    device: &Device,
    queue: &Queue,
    viewport: ScreenSize,
    output_texture: &TextureView,
    encoder: &mut CommandEncoder,
  ) {
    let full_output = self.context.end_frame();
    self.process_platform_output(window, full_output.platform_output);
    self.update_textures(device, queue, full_output.textures_delta);
    let clipped_primitives = self.tessellate(full_output.shapes, full_output.pixels_per_point);
    self.render(device, queue, clipped_primitives, viewport, output_texture, encoder);
  }
}

