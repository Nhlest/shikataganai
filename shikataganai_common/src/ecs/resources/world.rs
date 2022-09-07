use std::ops::Index;
use bevy::utils::hashbrown::HashMap;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crate::ecs::components::blocks::Block;
use crate::ecs::components::chunk::Chunk;
use crate::ecs::resources::light::LightLevel;
use crate::util::array::{ArrayIndex, DD, DDD};

#[derive(Default)]
pub struct GameWorld {
  // List of chunks that are being generated right now to avoid unnecessary matching on an enum for chunk access for 99.99% of runtime
  pub generating: Vec<DD>,
  pub chunks: HashMap<DD, Chunk>
}

impl GameWorld {
  pub fn remove_from_generating(&mut self, chunk_coord: DD) {
    if let Some(index) = self.generating.iter().position(|x| *x == chunk_coord) {
      self.generating.remove(index);
    }
  }

  pub fn get_chunk_coord(mut coord: DDD) -> DD {
    if coord.0 < 0 {
      coord.0 -= 15;
    }
    if coord.2 < 0 {
      coord.2 -= 15;
    }
    (coord.0 / 16, coord.2 / 16)
  }

  pub fn get(&self, c: DDD) -> Option<&Block> {
    let chunk_coord = Self::get_chunk_coord(c);
    self.chunks.get(&chunk_coord).map(|chunk| {
      if c.in_bounds(&chunk.grid.bounds) {
        Some(&chunk.grid[c])
      } else {
        None
      }
    }).flatten()
  }

  pub fn get_mut(&mut self, c: DDD) -> Option<&mut Block> {
    let chunk_coord = Self::get_chunk_coord(c);
    self.chunks.get_mut(&chunk_coord).map(|chunk| {
      &mut chunk.grid[c]
    })
  }

  pub fn get_light_level(&self, c: DDD) -> Option<LightLevel> {
    let chunk_coord = Self::get_chunk_coord(c);
    self.chunks.get(&chunk_coord).map(|chunk| {
      if c.in_bounds(&chunk.light_map.bounds) {
        Some(chunk.light_map[c])
      } else {
        None
      }
    }).flatten()
  }

  pub fn set_light_level(&mut self, c: DDD, light_level: LightLevel) -> Option<()> {
    let chunk_coord = Self::get_chunk_coord(c);
    self.chunks.get_mut(&chunk_coord).map(|chunk| {
      chunk.light_map[c] = light_level;
      ()
    })
  }
}