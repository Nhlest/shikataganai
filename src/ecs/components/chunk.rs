use bevy::prelude::*;

use crate::util::array::Array;

pub type BlockId = u8;

pub struct Block {
  pub block: BlockId,
  pub color: Color,
}

#[derive(Component)]
pub struct Chunk {
  pub grid: Array<Block>,
}

impl Chunk {
  pub fn new(size: (i32, i32, i32)) -> Self {
    Self {
      grid: Array::new_init(((0, 0, 0), size), |_| Block {
        block: 0,
        color: Color::RED,
      }),
    }
  }
}
