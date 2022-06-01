use crate::ecs::components::chunk::Chunk;
use crate::ecs::resources::chunk_map::ChunkMap;
use bevy::prelude::*;

pub fn light_system(
  // There be it
  chunk_map: Res<ChunkMap>,
  chunks: Query<&Chunk>,
) {
}
