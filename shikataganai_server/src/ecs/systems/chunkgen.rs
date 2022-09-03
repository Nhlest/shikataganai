use bevy::prelude::*;
use bevy::tasks::Task;
use bevy_renet::renet::RenetServer;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::networking::ServerChannel;
use shikataganai_common::util::array::DD;

#[derive(Component)]
pub struct ChunkTask {
  pub task: Task<Chunk>,
  pub coord: DD,
  pub client: u64
}

pub fn collect_async_chunks(
  mut query: Query<(Entity, &mut ChunkTask)>,
  mut commands: Commands,
  mut server: ResMut<RenetServer>
) {
  for (e, mut task) in query.iter_mut() {
    if let Some(chunk) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task.task)) {
      server.send_message(task.client, ServerChannel::ChunkTransfer.id(), bincode::serialize(&chunk).unwrap());
      // server.send_message(task.client, ServerChannel::ChunkTransfer.id(), bincode::serialize(&task.coord).unwrap());
      println!("Sent {:?} {:?}", task.coord, task.client);
      commands.entity(e).remove::<ChunkTask>();
    }
  }
}
