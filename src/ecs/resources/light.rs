use crate::util::array::{Array, Array3d};
use bevy::prelude::*;
use std::alloc::Layout;

pub struct LightLevel {
  pub heaven: u8,
  pub hearth: u8,
}

impl LightLevel {
  pub fn new(heaven: u8, hearth: u8) -> Self {
    Self {
      heaven,
      hearth
    }
  }
}