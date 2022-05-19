pub mod block;
pub mod chunk;
pub mod light;

use bevy::prelude::*;

#[derive(Component, Default, Copy, Clone, PartialEq, Debug)]
pub struct Location {
  pub x: i32,
  pub y: i32,
  pub z: i32,
}

impl Location {
  pub fn new(x: i32, y: i32, z: i32) -> Self {
    Self { x, y, z }
  }
}

impl Into<(i32, i32, i32)> for Location {
  fn into(self) -> (i32, i32, i32) {
    (self.x, self.y, self.z)
  }
}

impl From<Vec3> for Location {
  fn from(v: Vec3) -> Self {
    Self {
      x: v.x.floor() as i32,
      y: v.y.floor() as i32,
      z: v.z.floor() as i32,
    }
  }
}

impl From<&[i32; 3]> for Location {
  fn from([x, y, z]: &[i32; 3]) -> Self {
    Location { x: *x, y: *y, z: *z }
  }
}
