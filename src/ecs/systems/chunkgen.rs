use crate::ecs::plugins::voxel::RemeshEvent;
use crate::ecs::resources::chunk_map::{ChunkMap, ChunkTask};
use bevy::prelude::*;

pub fn collect_async_chunks(
  mut chunk_map: ResMut<ChunkMap>,
  mut query: Query<(Entity, &mut ChunkTask)>,
  mut commands: Commands,
  mut remesh: EventWriter<RemeshEvent>,
) {
  for (e, mut task) in query.iter_mut() {
    if let Some(chunk) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task.task)) {
      commands.entity(e).insert(chunk);
      let mut meta = chunk_map.map.get_mut(&task.coord).unwrap();
      meta.entity = Some(e);
      commands.entity(e).remove::<ChunkTask>();
      remesh.send(RemeshEvent::Remesh(task.coord));
    }
  }
}
