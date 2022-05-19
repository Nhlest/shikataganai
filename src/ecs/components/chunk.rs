use crate::ecs::components::block::{Block, BlockId};
use bevy::prelude::*;

use crate::util::array::Array;

#[derive(Component)]
pub struct Chunk {
  pub grid: Array<Block>,
}

impl Chunk {
  pub fn new<F: Fn((i32, i32, i32)) -> BlockId>(bounds: ((i32, i32, i32), (i32, i32, i32)), block_f: F) -> Self {
    Self {
      grid: Array::new_init(bounds, |c| Block { block: block_f(c) }),
    }
  }
}
