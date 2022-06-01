use crate::util::array::{Array, Array3d};
use bevy::prelude::*;
use std::alloc::Layout;

#[derive(Clone)]
pub struct LightMap {
  pub map: Array3d<u8>,
}

impl FromWorld for LightMap {
  fn from_world(world: &mut World) -> Self {
    Self {
      map: Array::new_zeroed((
        (0, 0, 0),
        ((5 - 1) * 16 - 1, 150, (5 - 1) * 16 - 1),
      )),
    }
  }
}

impl LightMap {
  pub fn zero_out(&mut self) {
    self.map.zero_out();
  }
}

pub struct SizedLightMap {
  pub ptr: *const u8,
  pub size: usize,
}

impl SizedLightMap {
  pub fn as_slice(&self) -> &[u8] {
    unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
  }
}

impl Drop for SizedLightMap {
  fn drop(&mut self) {
    unsafe {
      std::alloc::dealloc(self.ptr as *mut u8, Layout::from_size_align(self.size, 4).unwrap());
    }
  }
}

unsafe impl Send for SizedLightMap {}
unsafe impl Sync for SizedLightMap {}
