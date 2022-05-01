use std::fmt::Display;
use std::hash::Hash;
use std::ops::RangeInclusive;

use egui::{Align2, CollapsingHeader, CollapsingResponse, color_picker, ComboBox, Context, DragValue, Grid, InnerResponse, Response, Rgba, Ui, WidgetText, Window};
use egui::color_picker::Alpha;
use egui::emath::Numeric;
use ultraviolet::{Mat4, Vec2, Vec3, Vec4};

pub trait CtxRefWidgetsExt {
  fn window(&self, title: impl Into<WidgetText>, add_contents: impl FnOnce(&mut Ui)) -> Option<InnerResponse<Option<()>>>;
}

impl CtxRefWidgetsExt for &Context {
  #[inline]
  fn window(&self, title: impl Into<WidgetText>, add_contents: impl FnOnce(&mut Ui)) -> Option<InnerResponse<Option<()>>> {
    Window::new(title).show(self, add_contents)
  }
}

pub trait UiWidgetsExt {
  fn collapsing_open<R>(&mut self, heading: impl Into<WidgetText>, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R>;

  fn grid<R>(&mut self, id_source: impl Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;

  fn collapsing_with_grid<R>(&mut self, heading: impl Into<WidgetText>, grid_id: impl Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>>;
  fn collapsing_open_with_grid<R>(&mut self, heading: impl Into<WidgetText>, grid_id: impl Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>>;


  fn drag(&mut self, prefix: impl ToString, value: &mut impl Numeric, speed: impl Into<f64>) -> Response;
  fn drag_range<N: Numeric>(&mut self, prefix: impl ToString, value: &mut impl Numeric, speed: impl Into<f64>, clamp_range: RangeInclusive<N>) -> Response;
  fn drag_unlabelled(&mut self, value: &mut impl Numeric, speed: impl Into<f64>) -> Response;
  fn drag_unlabelled_range<N: Numeric>(&mut self, value: &mut impl Numeric, speed: impl Into<f64>, clamp_range: RangeInclusive<N>) -> Response;


  fn show_f32_2(&mut self, float: f32);

  fn show_f32_lp_5_2(&mut self, float: f32);
  fn show_f32_lp_7_2(&mut self, float: f32);

  fn show_prefixed_f32_lp_7_2(&mut self, prefix: impl Display, float: f32);


  fn show_vec2(&mut self, vec: &Vec2);
  fn drag_vec2(&mut self, speed: f32, vec: &mut Vec2);
  fn show_vec3(&mut self, vec: &Vec3);
  fn drag_vec3(&mut self, speed: f32, vec: &mut Vec3);
  fn show_vec4(&mut self, vec: &Vec4);
  fn show_vec4_unlabelled(&mut self, vec: &Vec4);
  fn drag_vec4(&mut self, speed: f32, vec: &mut Vec4);


  fn show_mat4(&mut self, mat: &Mat4);


  fn edit_color_vec4(&mut self, vec: &mut Vec4, alpha: Alpha);


  fn select_align2(&mut self, align: &mut Option<egui::Align2>);
}

impl UiWidgetsExt for Ui {
  #[inline]
  fn collapsing_open<R>(
    &mut self,
    heading: impl Into<WidgetText>,
    add_contents: impl FnOnce(&mut Ui) -> R,
  ) -> CollapsingResponse<R> {
    CollapsingHeader::new(heading).default_open(true).show(self, add_contents)
  }

  #[inline]
  fn grid<R>(&mut self, id_source: impl Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Grid::new(id_source).striped(true).show(self, add_contents)
  }

  #[inline]
  fn collapsing_with_grid<R>(&mut self, heading: impl Into<WidgetText>, grid_id: impl Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>> {
    self.collapsing(heading, |ui| { ui.grid(grid_id, add_contents) })
  }

  #[inline]
  fn collapsing_open_with_grid<R>(&mut self, heading: impl Into<WidgetText>, grid_id: impl Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<InnerResponse<R>> {
    self.collapsing_open(heading, |ui| { ui.grid(grid_id, add_contents) })
  }


  #[inline]
  fn drag(&mut self, prefix: impl ToString, value: &mut impl Numeric, speed: impl Into<f64>) -> Response {
    self.add(DragValue::new(value).prefix(prefix).speed(speed))
  }

  #[inline]
  fn drag_range<N: Numeric>(&mut self, prefix: impl ToString, value: &mut impl Numeric, speed: impl Into<f64>, clamp_range: RangeInclusive<N>) -> Response {
    self.add(DragValue::new(value).prefix(prefix).speed(speed).clamp_range(clamp_range))
  }

  #[inline]
  fn drag_unlabelled(&mut self, value: &mut impl Numeric, speed: impl Into<f64>) -> Response {
    self.add(DragValue::new(value).speed(speed))
  }

  #[inline]
  fn drag_unlabelled_range<N: Numeric>(&mut self, value: &mut impl Numeric, speed: impl Into<f64>, clamp_range: RangeInclusive<N>) -> Response {
    self.add(DragValue::new(value).speed(speed).clamp_range(clamp_range))
  }


  #[inline]
  fn show_f32_2(&mut self, float: f32) {
    self.monospace(format!("{:.2}", float));
  }


  #[inline]
  fn show_f32_lp_5_2(&mut self, float: f32) {
    self.monospace(format!("{:>5.2}", float));
  }

  #[inline]
  fn show_f32_lp_7_2(&mut self, float: f32) {
    self.monospace(format!("{:>7.2}", float));
  }


  #[inline]
  fn show_prefixed_f32_lp_7_2(&mut self, prefix: impl Display, float: f32) {
    self.monospace(format!("{}: {:>7.2}", prefix, float));
  }


  #[inline]
  fn show_vec2(&mut self, vec: &Vec2) {
    self.horizontal(|ui| {
      ui.show_prefixed_f32_lp_7_2("x: ", vec.x);
      ui.show_prefixed_f32_lp_7_2("y: ", vec.y);
    });
  }

  #[inline]
  fn drag_vec2(&mut self, speed: f32, vec: &mut Vec2) {
    self.horizontal(|ui| {
      ui.drag("x: ", &mut vec.y, speed);
      ui.drag("y: ", &mut vec.x, speed);
    });
  }

  #[inline]
  fn show_vec3(&mut self, vec: &Vec3) {
    self.horizontal(|ui| {
      ui.show_prefixed_f32_lp_7_2("x: ", vec.x);
      ui.show_prefixed_f32_lp_7_2("y: ", vec.y);
      ui.show_prefixed_f32_lp_7_2("z: ", vec.z);
    });
  }

  #[inline]
  fn drag_vec3(&mut self, speed: f32, vec: &mut Vec3) {
    self.horizontal(|ui| {
      ui.drag("x: ", &mut vec.x, speed);
      ui.drag("y: ", &mut vec.y, speed);
      ui.drag("z: ", &mut vec.z, speed);
    });
  }

  #[inline]
  fn show_vec4(&mut self, vec: &Vec4) {
    self.horizontal(|ui| {
      ui.show_prefixed_f32_lp_7_2("x: ", vec.x);
      ui.show_prefixed_f32_lp_7_2("y: ", vec.y);
      ui.show_prefixed_f32_lp_7_2("z: ", vec.z);
      ui.show_prefixed_f32_lp_7_2("w: ", vec.w);
    });
  }

  #[inline]
  fn show_vec4_unlabelled(&mut self, vec: &Vec4) {
    self.horizontal(|ui| {
      ui.show_f32_lp_7_2(vec.x);
      ui.show_f32_lp_7_2(vec.y);
      ui.show_f32_lp_7_2(vec.z);
      ui.show_f32_lp_7_2(vec.w);
    });
  }

  #[inline]
  fn drag_vec4(&mut self, speed: f32, vec: &mut Vec4) {
    self.horizontal(|ui| {
      ui.drag("x: ", &mut vec.x, speed);
      ui.drag("y: ", &mut vec.y, speed);
      ui.drag("z: ", &mut vec.z, speed);
      ui.drag("w: ", &mut vec.w, speed);
    });
  }


  #[inline]
  fn show_mat4(&mut self, mat: &Mat4) {
    self.vertical(|ui| {
      ui.show_vec4_unlabelled(&mat.cols[0]);
      ui.show_vec4_unlabelled(&mat.cols[1]);
      ui.show_vec4_unlabelled(&mat.cols[2]);
      ui.show_vec4_unlabelled(&mat.cols[3]);
    });
  }


  #[inline]
  fn edit_color_vec4(&mut self, vec: &mut Vec4, alpha: Alpha) {
    let mut color = Rgba::from_rgba_premultiplied(vec.x, vec.y, vec.z, vec.w).into();
    color_picker::color_edit_button_srgba(self, &mut color, alpha);
    let color: Rgba = color.into();
    *vec = Vec4::new(color.r(), color.g(), color.b(), color.a());
  }


  #[inline]
  fn select_align2(&mut self, anchor: &mut Option<Align2>) {
    ComboBox::from_id_source("Anchor")
      .selected_text(match anchor {
        None => "None",
        Some(Align2::LEFT_TOP) => "Left Top",
        Some(Align2::LEFT_BOTTOM) => "Left Bottom",
        Some(Align2::RIGHT_TOP) => "Right Top",
        Some(Align2::RIGHT_BOTTOM) => "Right Bottom",
        Some(_) => "Other",
      })
      .show_ui(self, |ui| {
        ui.selectable_value(anchor, None, "None");
        ui.selectable_value(anchor, Some(Align2::LEFT_TOP), "Left Top");
        ui.selectable_value(anchor, Some(Align2::LEFT_BOTTOM), "Left Bottom");
        ui.selectable_value(anchor, Some(Align2::RIGHT_TOP), "Right Top");
        ui.selectable_value(anchor, Some(Align2::RIGHT_BOTTOM), "Right Bottom");
      });
  }
}
