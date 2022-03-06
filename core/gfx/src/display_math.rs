use std::fmt::{Display, Formatter};

use ultraviolet::{UVec3, Vec3};

pub struct DisplayUVec3(UVec3);

impl Display for DisplayUVec3 {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str("(x:")?;
    self.0.x.fmt(f)?;
    f.write_str(" y:")?;
    self.0.y.fmt(f)?;
    f.write_str(" z:")?;
    self.0.z.fmt(f)?;
    f.write_str(")")
  }
}

pub trait UVec3DisplayExt {
  fn display(self) -> DisplayUVec3;
}

impl UVec3DisplayExt for UVec3 {
  fn display(self) -> DisplayUVec3 {
    DisplayUVec3(self)
  }
}


pub struct DisplayVec3(Vec3);

impl Display for DisplayVec3 {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str("(x:")?;
    self.0.x.fmt(f)?;
    f.write_str(" y:")?;
    self.0.y.fmt(f)?;
    f.write_str(" z:")?;
    self.0.z.fmt(f)?;
    f.write_str(")")
  }
}

pub trait Vec3DisplayExt {
  fn display(self) -> DisplayVec3;
}

impl Vec3DisplayExt for Vec3 {
  fn display(self) -> DisplayVec3 {
    DisplayVec3(self)
  }
}
