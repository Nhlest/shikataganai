use bevy::prelude::*;
use noise::{Perlin, Seedable, NoiseFn};

use crate::ecs::components::blocks::Block;
use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::resources::light::LightLevel;
use crate::util::array::{Array, Array2d, Array3d, Bounds, DD, DDD};

pub const CHUNK_MAX_HEIGHT: i32 = 127;

#[derive(Component)]
pub struct Chunk {
  pub grid: Array3d<Block>,
  pub light_map: Array3d<LightLevel>,
}

fn noise(perlin: &Perlin, c: DDD) -> f64 {
  let a = perlin.get([c.0 as f64 / 20.0, 0.0, c.2 as f64 / 20.0]);
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
    let from = (coord.0 * 16, 0, coord.1 * 16);
    let to = (coord.0 * 16 + 15, CHUNK_MAX_HEIGHT, coord.1 * 16 + 15);
    let perlin_top = Perlin::new().set_seed(12);
    let v = Array2d::new_init(((from.0, from.2), (to.0, to.2)), |(x, z)| noise(&perlin, (x, 0, z)));
    let vtop = Array2d::new_init(((from.0, from.2), (to.0, to.2)), |(x, z)| noise(&perlin_top, (x, 0, z)));

    Chunk::new((from, to), |(x, y, z)| {
      let bottom = v[(x, z)];
      let top = vtop[(x, z)];

      let bottom_extent = (bottom * 30.0).floor() as i32;
      let top_extent = ((top + 1.0) * bottom / 2.0 * 30.0).floor() as i32;

      if bottom <= 0.0 {
        BlockId::Air
      } else if y < 30 {
        if (30 - y) < bottom_extent {
          BlockId::Cobble
        } else {
          BlockId::Air
        }
      } else {
        if (y - 30) > top_extent {
          BlockId::Air
        } else if (y - 30) == top_extent {
          BlockId::Grass
        } else if y - 28 >= top_extent {
          BlockId::Dirt
        } else {
          BlockId::Cobble
        }
      }
    })
  }
}
