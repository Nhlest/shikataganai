use bevy::prelude::*;

use crate::util::array::Array;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BlockId {
  Air,
  Dirt,
  Grass,
  Cobble,
}

impl BlockId {
  pub fn into_array_of_faces(self) -> [u16; 6] {
    match self {
      BlockId::Air => [0, 0, 0, 0, 0, 0],
      BlockId::Dirt => [1, 1, 1, 1, 1, 1],
      BlockId::Grass => [2, 2, 2, 2, 3, 1],
      BlockId::Cobble => [4, 4, 4, 4, 4, 4],
    }
  }
}

pub struct Block {
  pub block: BlockId,
  pub color: Color,
}

#[derive(Component)]
pub struct Chunk {
  pub grid: Array<Block>,
}

impl Chunk {
  pub fn new<F: Fn((i32, i32, i32)) -> BlockId>(bounds: ((i32, i32, i32), (i32, i32, i32)), block_f: F) -> Self {
    Self {
      grid: Array::new_init(bounds, |c| Block {
        block: block_f(c),
        color: Color::RED,
      }),
    }
  }
}
