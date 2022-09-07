use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_renet::renet::RenetClient;
use shikataganai_common::ecs::components::blocks::Block;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::networking::{ClientChannel, PlayerCommand};
use shikataganai_common::util::array::{ImmediateNeighbours, DD, DDD};
use std::mem::MaybeUninit;
use shikataganai_common::ecs::resources::world::GameWorld;
use crate::ecs::plugins::client::send_message;

pub trait ClientGameWorld {
  fn get_chunk_or_request(&mut self, chunk_coord: DD, client: &mut RenetClient) -> Option<&Chunk>;
  fn get_chunk_or_request_mut(&mut self, chunk_coord: DD, client: &mut RenetClient) -> Option<&mut Chunk>;
  fn get_block_or_request(&mut self, coord: DDD, client: &mut RenetClient) -> Option<&Block>;
  fn get_block_or_request_mut(&mut self, coord: DDD, client: &mut RenetClient) -> Option<&mut Block>;
}

impl ClientGameWorld for GameWorld {
  fn get_chunk_or_request(&mut self, chunk_coord: DD, client: &mut RenetClient) -> Option<&Chunk> {
    match self.chunks.get(&chunk_coord) {
      None => {
        if !self.generating.contains(&chunk_coord) {
          self.generating.push(chunk_coord);
          send_message(client, PlayerCommand::RequestChunk { chunk_coord });
        }
        None
      }
      Some(chunk) => Some(chunk)
    }
  }

  fn get_chunk_or_request_mut(&mut self, chunk_coord: DD, client: &mut RenetClient) -> Option<&mut Chunk> {
    match self.chunks.get_mut(&chunk_coord) {
      None => {
        if !self.generating.contains(&chunk_coord) {
          self.generating.push(chunk_coord);
          send_message(client, PlayerCommand::RequestChunk { chunk_coord });
        }
        None
      }
      Some(chunk) => Some(chunk)
    }
  }

  fn get_block_or_request(&mut self, coord: DDD, client: &mut RenetClient) -> Option<&Block> {
    let chunk_coord = GameWorld::get_chunk_coord(coord);
    self.get_chunk_or_request(chunk_coord, client).map(|chunk| {
      &chunk.grid[coord]
    })
  }

  fn get_block_or_request_mut(&mut self, coord: DDD, client: &mut RenetClient) -> Option<&mut Block> {
    let chunk_coord = GameWorld::get_chunk_coord(coord);
    self.get_chunk_or_request_mut(chunk_coord, client).map(|chunk| {
      &mut chunk.grid[coord]
    })
  }
}

// fn get_many_mut<const N: usize>(&mut self, cs: [DDD; N]) -> Option<[&mut Block; N]> {
//   for i in 0..N {
//     for j in 0..i {
//       if cs[i] == cs[j] {
//         return None;
//       }
//     }
//   }
//   let mut chunk_entities: [Entity; N] = unsafe { MaybeUninit::uninit().assume_init() };
//   for i in 0..N {
//     let c = cs[i];
//     if c.1 < 0 || c.1 > 255 {
//       return None;
//     }
//     match self.get_chunk_entity_or_queue(c) {
//       None => return None,
//       Some(entity) => {
//         chunk_entities[i] = entity;
//       }
//     }
//   }
//   Some(
//     chunk_entities
//       .map(|e| unsafe { self.chunks.get_unchecked(e).unwrap() })
//       .into_iter()
//       .enumerate()
//       .map(|(i, c)| &mut c.into_inner().grid[cs[i]])
//       .collect::<Vec<_>>()
//       .try_into()
//       .unwrap(),
//   )
// }

//   fn propagate_light(&mut self, c: DDD, remesh: &mut HashSet<DD>) {
//     let mut queue = vec![c];
//     while !queue.is_empty() {
//       let c = queue.pop().unwrap();
//       if self.get_single(c).map_or(false, |e| e.visible()) {
//         self.set_light_level(c, LightLevel::dark());
//         remesh.insert(ChunkMap::get_chunk_coord(c));
//         continue;
//       }
//       if let Some(current_light) = self.get_light_level(c) {
//         let mut new_heaven_light = None;
//         let mut new_hearth_light = None;
//         for heaven_check in c.immediate_neighbours() {
//           if let Some(LightLevel { mut heaven, hearth }) = self.get_light_level(heaven_check) {
//             if heaven_check.1 - c.1 == 1 && heaven == 15 {
//               heaven += 1
//             }
//             // TODO: fix this clusterfuck
//             let new = if heaven - 1 > 16 { 0 } else { heaven - 1 };
//             if current_light.heaven < heaven - 1 && heaven > 0 && new_heaven_light.map_or(true, |x| new > x) {
//               new_heaven_light = Some(new);
//             }
//             let new = if hearth - 1 > 16 { 0 } else { hearth - 1 };
//             if current_light.hearth < hearth - 1 && hearth > 0 && new_hearth_light.map_or(true, |x| new > x) {
//               new_hearth_light = Some(new);
//             }
//           }
//         }
//         if new_heaven_light.is_none() && new_hearth_light.is_none() {
//           continue;
//         }
//         let new_light = LightLevel::new(
//           new_heaven_light.unwrap_or(current_light.heaven),
//           new_hearth_light.unwrap_or(current_light.hearth),
//         );
//         self.set_light_level(c, new_light);
//         let chunk_coord = ChunkMap::get_chunk_coord(c);
//         remesh.insert(chunk_coord);
//         for i in c.immediate_neighbours() {
//           if let Some(LightLevel { heaven, hearth }) = self.get_light_level(i) {
//             if (heaven >= new_light.heaven - 1 || new_light.heaven == 0)
//               && (hearth >= new_light.hearth - 1 || new_light.hearth == 0)
//             {
//               continue;
//             }
//           }
//           remesh.insert(ChunkMap::get_chunk_coord(i));
//           queue.push(i);
//         }
//       }
//     }
//   }
// }