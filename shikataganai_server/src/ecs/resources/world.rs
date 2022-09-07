use std::io::Write;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_renet::renet::RenetServer;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::networking::{RELIABLE_CHANNEL_MAX_LENGTH, ServerChannel, ServerMessage};
use shikataganai_common::util::array::DD;
use crate::ecs::systems::chunkgen::ChunkTask;

pub trait ServerGameWorld {
  fn get_chunk_or_spawn(&mut self, chunk_coord: DD, commands: &mut Commands, client: u64) -> Option<&Chunk>;
}

impl ServerGameWorld for GameWorld {
  fn get_chunk_or_spawn(&mut self, chunk_coord: DD, commands: &mut Commands, client: u64) -> Option<&Chunk> {
    match self.chunks.get(&chunk_coord) {
      None => {
        if !self.generating.contains(&chunk_coord) {
          self.generating.push(chunk_coord);
          let dispatcher = AsyncComputeTaskPool::get();
          commands.spawn().insert(ChunkTask {
            task: dispatcher.spawn(Chunk::generate(chunk_coord)),
            coord: chunk_coord,
            client,
          });
        }
        None
      }
      Some(chunk) => Some(chunk)
    }
  }
}

pub fn send_chunk_data(server: &mut RenetServer, chunk: &Chunk, client: u64) {
  let data = bincode::serialize(&chunk).unwrap();
  let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
  encoder.write_all(&data).unwrap();
  let message = encoder.finish().unwrap();
  assert!(message.len() <= RELIABLE_CHANNEL_MAX_LENGTH as usize, "Chunk packet size limit reached. Stopgap has been used up. Good luck fixing that.");
  server.send_message(
    client,
    ServerChannel::GameEvent.id(),
    bincode::serialize(&ServerMessage::ChunkData { chunk: message }).unwrap(),
  );
}
