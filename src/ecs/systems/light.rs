use crate::ecs::components::block::BlockId;
use crate::ecs::components::chunk::Chunk;
use crate::ecs::components::light::LightSource;
use crate::ecs::components::Location;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::ecs::resources::light::{LightMap, Relight};
use bevy::prelude::*;

fn flood_fill(
  chunk_map: &Res<ChunkMap>,
  chunks: &Query<&Chunk>,
  light_map: &mut ResMut<LightMap>,
  location: Location,
  level: u8,
) {
  if level == 0 {
    return;
  }
  if !light_map.map.in_bounds(location.into()) {
    return;
  } // TODO: move to chunk map
  if light_map.map[location.into()] >= level {
    return;
  }
  if let Some((e, c)) = chunk_map.get_path_to_block_location(location) {
    if chunks.get(e).unwrap().grid[c.into()].block != BlockId::Air {
      return;
    }
  }

  light_map.map[location.into()] = level;
  for ix in -1..=1 {
    for iy in -1..=1 {
      for iz in -1..=1 {
        if (ix as i32).abs() + (iy as i32).abs() + (iz as i32).abs() != 1 {
          continue;
        }
        let new_loc = Location::new(location.x + ix, location.y + iy, location.z + iz);
        flood_fill(chunk_map, chunks, light_map, new_loc, level - 1);
      }
    }
  }
}

pub fn light_system(
  // There be it
  chunk_map: Res<ChunkMap>,
  chunks: Query<&Chunk>,
  mut light_map: ResMut<LightMap>,
  light_sources: Query<&Location, With<LightSource>>,
  mut relight: ResMut<Relight>,
) {
  if !relight.0 {
    return;
  }
  relight.0 = false;
  light_map.zero_out();
  for light in light_sources.iter() {
    flood_fill(&chunk_map, &chunks, &mut light_map, light.clone(), 21)
  }
}
