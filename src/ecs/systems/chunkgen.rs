use crate::ecs::components::chunk::ChunkTask;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::RemeshEvent;
use crate::ecs::resources::chunk_map::ChunkMap;
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
      for i in task.coord.0 - 1..=task.coord.0 + 1 {
        for j in task.coord.1 - 1..=task.coord.1 + 1 {
          remesh.send(RemeshEvent::Remesh((i, j)));
        }
      }
    }
  }
}
