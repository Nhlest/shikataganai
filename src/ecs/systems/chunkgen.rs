use crate::ecs::plugins::voxel::RemeshEvent;
use crate::ecs::resources::chunk_map::{ChunkMap, ChunkMeta, ChunkTask};
use bevy::prelude::*;

pub fn collect_async_chunks(
  mut chunk_map: ResMut<ChunkMap>,
  mut query: Query<(Entity, &mut ChunkTask)>,
  mut commands: Commands,
  mut remesh: EventWriter<RemeshEvent>,
) {
  for (e, mut task) in query.iter_mut() {
    if let Some((chunk, coord)) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task.task)) {
      commands.entity(e).insert(chunk);
      chunk_map.map.insert(coord, ChunkMeta::new(e));
      commands.entity(e).remove::<ChunkTask>();
      remesh.send(RemeshEvent::Remesh(coord));
    }
  }
}
