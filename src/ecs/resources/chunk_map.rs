use crate::ecs::components::chunk::{BlockId, Chunk};
use crate::ecs::components::Location;
use crate::util::array::Array2d;
use bevy::prelude::*;

pub struct ChunkMeta {
  pub entity: Entity,
  pub bounds: ((i32, i32, i32), (i32, i32, i32)),
  pub size_of_a_voxel: f32,
  pub lower_left_back_coord: Vec3,
}

pub struct ChunkMap {
  pub map: Array2d<ChunkMeta>,
}

#[derive(Clone)]
pub struct ChunkMapSize {
  pub x: i32,
  pub y: i32,
}

impl Default for ChunkMapSize {
  fn default() -> Self {
    Self { x: 5, y: 5 }
  }
}

impl FromWorld for ChunkMap {
  fn from_world(world: &mut World) -> Self {
    let chunk_map_size = world.get_resource::<ChunkMapSize>().cloned().unwrap_or_default();
    let chunk_entities = Array2d::new_init(((0, 0), (chunk_map_size.x - 1, chunk_map_size.y - 1)), |(x, y)| {
      let bounds = ((x * 16, 0, y * 16), ((x + 1) * 16 - 1, 150, (y + 1) * 16 - 1));
      let entity = world
        .spawn()
        .insert(Chunk::new(bounds, {
          |(x, y, z)| {
            if y + z + x > 150 {
              BlockId::Air
            } else {
              if y % 2 == 0 {
                BlockId::Cobble
              } else {
                BlockId::Grass
              }
            }
          }
        }))
        .id();
      ChunkMeta {
        entity,
        bounds,
        size_of_a_voxel: 1.0,
        lower_left_back_coord: Vec3::new(x as f32 * 16.0, 0.0, y as f32 * 16.0),
      }
    });
    ChunkMap { map: chunk_entities }
  }
}

impl ChunkMap {
  pub fn get_path_to_block(&self, coord: Vec3) -> Option<(Entity, Location)> {
    self.get_path_to_block_location(Location::from(coord))
  }

  pub fn get_path_to_block_location(&self, coord: Location) -> Option<(Entity, Location)> {
    let chunk_coord_i = (coord.x / 16, coord.z / 16);
    if !self.map.in_bounds(chunk_coord_i) {
      return None;
    }
    let chunk = &self.map[chunk_coord_i];
    let block_coord = Location::new(coord.x, coord.y, coord.z);
    let ((x0, y0, z0), (x1, y1, z1)) = chunk.bounds;
    if x0 <= block_coord.x
      && x1 >= block_coord.x
      && y0 <= block_coord.y
      && y1 >= block_coord.y
      && z0 <= block_coord.z
      && z1 >= block_coord.z
    {
      Some((chunk.entity, block_coord))
    } else {
      None
    }
  }
}
