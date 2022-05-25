use crate::ecs::plugins::voxel::Remesh;
use crate::ecs::resources::chunk_map::{ChunkMap, ChunkTask};
use crate::ecs::resources::light::Relight;
use bevy::prelude::*;

pub fn collect_async_chunks(
  mut chunk_map: ResMut<ChunkMap>,
  mut query: Query<(Entity, &mut ChunkTask)>,
  mut commands: Commands,
  mut remesh: ResMut<Remesh>,
  mut relight: ResMut<Relight>,
) {
  for (e, mut task) in query.iter_mut() {
    if let Some((chunk, coord)) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task.task)) {
      commands.entity(e).insert(chunk);
      chunk_map.map.get_mut(&coord).unwrap().generated = true;
      commands.entity(e).remove::<ChunkTask>();
      remesh.remesh();
      relight.relight();
    }
  }
}
