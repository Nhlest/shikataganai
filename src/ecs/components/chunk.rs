use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use crate::ecs::components::block::{Block, BlockData, BlockId};
use bevy::prelude::*;
use crate::ecs::resources::light::LightLevel;

use crate::util::array::{Array, Array3d, Bounds, DD, DDD};

#[derive(Component)]
pub struct Chunk {
  pub grid: Array3d<Block>,
  pub light_map: Array3d<LightLevel>,
}

impl Chunk {
  pub fn new<F: Fn(DDD) -> BlockId>(bounds: Bounds<DDD>, block_f: F) -> Self {
    Self {
      grid: Array::new_init(bounds, |c| Block::new(block_f(c))),
      light_map: Array::new_init(bounds, |c| LightLevel::new(0, 0)),
    }
  }

  pub async fn generate(coord: DD) -> (Chunk, DD) {
    let from = (coord.0 * 16, 0, coord.1 * 16);
    let to = (coord.0 * 16 + 15, 255, coord.1 * 16 + 15);
    (
      Chunk::new(
        (from, to),
        |(_, y, _)| {
          if y < 14 {
            BlockId::Cobble
          } else {
            BlockId::Air
          }
        },
      ),
      coord,
    )
  }
}
