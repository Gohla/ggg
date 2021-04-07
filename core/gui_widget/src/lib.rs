use std::fmt::Display;
use std::hash::Hash;

use egui::{CollapsingHeader, CollapsingResponse, CtxRef, DragValue, Grid, InnerResponse, Response, Ui, Window};
use egui::emath::Numeric;
use ultraviolet::{Mat4, Vec2, Vec3, Vec4};

pub trait CtxRefWidgetsExt {
  fn window(&self, title: impl Into<String>, add_contents: impl FnOnce(&mut Ui)) -> Option<Response>;
}

impl CtxRefWidgetsExt for CtxRef {
  #[inline]
  fn window(&self, title: impl Into<String>, add_contents: impl FnOnce(&mut Ui)) -> Option<Response> {
    Window::new(title).show(self, add_contents)
  }
}

pub trait UiWidgetsExt {
  fn collapsing_open<R>(&mut self, heading: impl Into<String>, add_contents: impl FnOnce(&mut Ui) -> R) -> CollapsingResponse<R>;

  fn grid<R>(&mut self, id_source: impl Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R>;

  fn drag(&mut self, prefix: impl ToString, value: &mut impl Numeric, speed: impl Into<f64>) -> Response;

  fn drag_vec2(&mut self, speed: f32, vec: &mut Vec2);
  fn drag_vec3(&mut self, speed: f32, vec: &mut Vec3);
  fn drag_vec4(&mut self, speed: f32, vec: &mut Vec4);

  fn show_f32(&mut self, float: f32);
  fn show_prefixed_f32(&mut self, prefix: impl Display, float: f32);

  fn show_vec2(&mut self, vec: &Vec2);
  fn show_vec3(&mut self, vec: &Vec3);
  fn show_vec4(&mut self, vec: &Vec4);

  fn show_vec4_unlabelled(&mut self, vec: &Vec4);

  fn show_mat4(&mut self, mat: &Mat4);
}

impl UiWidgetsExt for Ui {
  #[inline]
  fn collapsing_open<R>(
    &mut self,
    heading: impl Into<String>,
    add_contents: impl FnOnce(&mut Ui) -> R,
  ) -> CollapsingResponse<R> {
    CollapsingHeader::new(heading).default_open(true).show(self, add_contents)
  }


  #[inline]
  fn grid<R>(&mut self, id_source: impl Hash, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    Grid::new(id_source).striped(true).show(self, add_contents)
  }


  #[inline]
  fn drag(&mut self, prefix: impl ToString, value: &mut impl Numeric, speed: impl Into<f64>) -> Response {
    self.add(DragValue::new(value).prefix(prefix).speed(speed))
  }


  #[inline]
  fn drag_vec2(&mut self, speed: f32, vec: &mut Vec2) {
    self.horizontal(|ui| {
      ui.drag("x: ", &mut vec.y, speed);
      ui.drag("y: ", &mut vec.x, speed);
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
  fn drag_vec4(&mut self, speed: f32, vec: &mut Vec4) {
    self.horizontal(|ui| {
      ui.drag("x: ", &mut vec.x, speed);
      ui.drag("y: ", &mut vec.y, speed);
      ui.drag("z: ", &mut vec.z, speed);
      ui.drag("w: ", &mut vec.w, speed);
    });
  }


  #[inline]
  fn show_f32(&mut self, float: f32) {
    self.monospace(format!("{:>8.2}", float));
  }

  #[inline]
  fn show_prefixed_f32(&mut self, prefix: impl Display, float: f32) {
    self.monospace(format!("{}: {:>8.2}", prefix, float));
  }


  #[inline]
  fn show_vec2(&mut self, vec: &Vec2) {
    self.horizontal(|ui| {
      ui.show_prefixed_f32("x: ", vec.x);
      ui.show_prefixed_f32("y: ", vec.y);
    });
  }

  #[inline]
  fn show_vec3(&mut self, vec: &Vec3) {
    self.horizontal(|ui| {
      ui.show_prefixed_f32("x: ", vec.x);
      ui.show_prefixed_f32("y: ", vec.y);
      ui.show_prefixed_f32("z: ", vec.z);
    });
  }

  #[inline]
  fn show_vec4(&mut self, vec: &Vec4) {
    self.horizontal(|ui| {
      ui.show_prefixed_f32("x: ", vec.x);
      ui.show_prefixed_f32("y: ", vec.y);
      ui.show_prefixed_f32("z: ", vec.z);
      ui.show_prefixed_f32("w: ", vec.w);
    });
  }


  fn show_vec4_unlabelled(&mut self, vec: &Vec4) {
    self.horizontal(|ui| {
      ui.show_f32(vec.x);
      ui.show_f32(vec.y);
      ui.show_f32(vec.z);
      ui.show_f32(vec.w);
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
}
