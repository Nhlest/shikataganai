use crate::ecs::components::block::{Block, BlockId};
use crate::ecs::resources::light::LightLevel;
use bevy::prelude::*;
use bevy::tasks::Task;

use crate::util::array::{Array, Array3d, Bounds, DD, DDD};

const CHUNK_MAX_HEIGHT: i32 = 255;

#[derive(Component)]
pub struct ChunkTask {
  pub task: Task<Chunk>,
  pub coord: DD,
}

#[derive(Component)]
pub struct Chunk {
  pub grid: Array3d<Block>,
  pub light_map: Array3d<LightLevel>,
}

impl Chunk {
  pub fn new<F: Fn(DDD) -> BlockId>(bounds: Bounds<DDD>, block_f: F) -> Self {
    let mut chunk = Self {
      grid: Array::new_init(bounds, |c| Block::new(block_f(c))),
      light_map: Array::new_init(bounds, |_| LightLevel::new(0, 0)),
    };
    for ix in bounds.0 .0..=bounds.1 .0 {
      for iz in bounds.0 .2..=bounds.1 .2 {
        for iy in (0..=CHUNK_MAX_HEIGHT).rev() {
          if chunk.grid[(ix, iy, iz)].visible() {
            break;
          }
          chunk.light_map[(ix, iy, iz)] = LightLevel::new(16, 0);
        }
      }
    }
    chunk
  }

  pub async fn generate(coord: DD) -> Chunk {
    let from = (coord.0 * 16, 0, coord.1 * 16);
    let to = (coord.0 * 16 + 15, CHUNK_MAX_HEIGHT, coord.1 * 16 + 15);
    Chunk::new((from, to), |(_, y, _)| {
      if y < 30 {
        BlockId::Cobble
      } else if y < 35 {
        BlockId::Dirt
      } else if y < 36 {
        BlockId::Grass
      } else {
        BlockId::Air
      }
    })
  }
}
