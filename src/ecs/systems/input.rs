use crate::ecs::components::chunk::{BlockId, Chunk};
use crate::ecs::components::Location;
use crate::ecs::plugins::camera::Selection;
use crate::ecs::plugins::voxel::Remesh;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::ecs::systems::light::Relight;
use bevy::prelude::*;

pub fn block_input(
  mouse: Res<Input<MouseButton>>,
  selection: Res<Option<Selection>>,
  mut chunks: Query<&mut Chunk>,
  chunk_map: Res<ChunkMap>,
  mut relight: ResMut<Relight>,
  mut remesh: ResMut<Remesh>,
) {
  match selection.into_inner() {
    None => {}
    Some(Selection { cube, face }) => {
      if mouse.just_pressed(MouseButton::Left) {
        if let Some((e, c)) = chunk_map.get_path_to_block_location(Location::new(cube[0], cube[1], cube[2])) {
          chunks.get_mut(e).unwrap().grid[c.into()].block = BlockId::Air;
          relight.0 = true;
          remesh.0 = true;
        }
      }
      if mouse.just_pressed(MouseButton::Right) {
        if let Some((e, c)) = chunk_map.get_path_to_block_location(Location::new(face[0], face[1], face[2])) {
          chunks.get_mut(e).unwrap().grid[c.into()].block = BlockId::Dirt;
          relight.0 = true;
          remesh.0 = true;
        }
      }
    }
  }
}
