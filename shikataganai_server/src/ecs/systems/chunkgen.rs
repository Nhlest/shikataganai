use crate::ecs::resources::world::send_chunk_data;
use bevy::prelude::*;
use bevy::tasks::Task;
use bevy_renet::renet::RenetServer;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::networking::{ServerChannel, ServerMessage};
use shikataganai_common::util::array::DD;
use std::io::Write;

#[derive(Component)]
pub struct ChunkTask {
  pub task: Task<Chunk>,
  pub coord: DD,
  pub client: u64,
}

pub fn collect_async_chunks(
  mut query: Query<(Entity, &mut ChunkTask)>,
  mut commands: Commands,
  mut server: ResMut<RenetServer>,
  mut world: ResMut<GameWorld>,
) {
  for (e, mut task) in query.iter_mut() {
    if let Some(chunk) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task.task)) {
      send_chunk_data(server.as_mut(), &chunk, task.client);
      world.chunks.insert(task.coord, chunk);
      world.remove_from_generating(task.coord);
      commands.entity(e).remove::<ChunkTask>();
    }
  }
}
