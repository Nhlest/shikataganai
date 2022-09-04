use bevy::prelude::*;
use bevy::tasks::Task;
use bevy_renet::renet::RenetServer;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use shikataganai_common::ecs::components::chunk::Chunk;
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
) {
  for (e, mut task) in query.iter_mut() {
    if let Some(chunk) = futures_lite::future::block_on(futures_lite::future::poll_once(&mut task.task)) {
      let data = bincode::serialize(&chunk).unwrap();
      let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
      encoder.write_all(&data).unwrap();
      let message = encoder.finish().unwrap();
      println!("Sent {:?} {:?} LEN: {}", task.coord, task.client, message.len());
      server.send_message(
        task.client,
        ServerChannel::GameEvent.id(),
        bincode::serialize(&ServerMessage::ChunkData { chunk: message }).unwrap(),
      );
      // server.send_message(task.client, ServerChannel::ChunkTransfer.id(), bincode::serialize(&task.coord).unwrap());
      commands.entity(e).remove::<ChunkTask>();
    }
  }
}
