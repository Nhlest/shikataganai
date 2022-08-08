use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::components::blocks::Block;
use crate::ecs::resources::light::LightLevel;
use bevy::prelude::*;
use bevy::tasks::Task;
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use noise::*;

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

fn noise(plane: &NoiseMap, c: DDD) -> f64 {
  let a = plane.get_value(c.0 as usize, c.2 as usize);
  a
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
    let perlin = Perlin::new().set_seed(11);
    let plane = PlaneMapBuilder::new(&perlin)
      .set_x_bounds(-2.0, 2.0)
      .set_y_bounds(-2.0, 2.0)
      .build();
    let from = (coord.0 * 16, 0, coord.1 * 16);
    let to = (coord.0 * 16 + 15, CHUNK_MAX_HEIGHT, coord.1 * 16 + 15);
    Chunk::new((from, to), |(x, y, z)| {
      let a = noise(&plane, (x, y, z));
      if (((a + 1.0) * 15.0) as i32 + 15) < y {
        BlockId::Air
      } else if (((a + 1.0) * 15.0) as i32 + 15) < y + 1 {
        BlockId::Grass
      } else if (((a + 1.0) * 15.0) as i32 + 15) < y + 3 {
        BlockId::Dirt
      } else {
        BlockId::Cobble
      }
    })
  }
}
