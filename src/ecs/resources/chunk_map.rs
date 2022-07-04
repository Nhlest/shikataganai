use std::cell::Cell;
use crate::ecs::components::block::{Block, BlockId};
use crate::ecs::components::chunk::{Chunk};
use bevy::ecs::system::{SystemMeta, SystemParam, SystemParamFetch, SystemParamState};
use crate::util::array::{DD, DDD};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;
use std::mem::MaybeUninit;
use bevy::ecs::query::{Fetch, QueryItem, ReadOnlyWriteFetch, WorldQuery, WorldQueryGats, WriteFetch, WriteState};
use crate::ecs::resources::light::LightLevel;
use duplicate::{duplicate, duplicate_item};

pub struct ChunkMeta {
  pub entity: Entity,
}

impl ChunkMeta {
  pub fn new(entity: Entity) -> Self {
    Self {
      entity,
    }
  }
}

#[derive(SystemParam)]
pub struct BlockAccessorSpawner<'w, 's> {
  pub chunk_map: ResMut<'w, ChunkMap>,
  pub chunks: Query<'w, 's, &'static mut Chunk>,
  pub commands: Commands<'w, 's>,
  pub dispatcher: Res<'w, AsyncComputeTaskPool>
}

#[derive(SystemParam)]
pub struct BlockAccessorStatic<'w, 's> {
  pub chunk_map: ResMut<'w, ChunkMap>,
  pub chunks: Query<'w, 's, &'static mut Chunk>,
}

pub trait BlockAccessorInternal<'w, 's> {
  fn get_chunk_entity_or_queue(&mut self, c: DDD) -> Option<Entity>;
}

impl<'w, 's> BlockAccessorInternal<'w, 's> for BlockAccessorStatic<'w, 's> {
  fn get_chunk_entity_or_queue(&mut self, c: DDD) -> Option<Entity> {
    let chunk_coord = ChunkMap::get_chunk_coord(c);
    match self.chunk_map.map.get(&chunk_coord) {
      None => {
        None
      }
      Some(ChunkMeta { entity }) => {
        Some(*entity)
      }
    }
  }
}

impl<'w, 's> BlockAccessorInternal<'w, 's> for BlockAccessorSpawner<'w, 's> {
  fn get_chunk_entity_or_queue(&mut self, c: DDD) -> Option<Entity> {
    let chunk_coord = ChunkMap::get_chunk_coord(c);
    match self.chunk_map.map.get(&chunk_coord) {
      None => {
        let task = self.dispatcher.spawn(Chunk::generate(chunk_coord));
        self.commands.spawn().insert(ChunkTask { task });
        None
      }
      Some(ChunkMeta { entity }) => {
        Some(*entity)
      }
    }
  }
}

pub trait BlockAccessor {
  fn get_single(&mut self, c: DDD) -> Option<&Block>;
  fn get_mut(&mut self, c: DDD) -> Option<&mut Block>;
  fn get_many_mut<const N: usize>(&mut self, cs: [DDD; N]) -> Option<[&mut Block; N]>;
}

#[duplicate_item(T; [BlockAccessorSpawner]; [BlockAccessorStatic])]
impl<'w, 's> BlockAccessor for T<'w, 's> {
  fn get_single(&mut self, c: DDD) -> Option<&Block> {
    if c.1 < 0 || c.1 > 255 {
      return None;
    }
    self
      .get_chunk_entity_or_queue(c)
      .map(move |entity| &self.chunks.get(entity).unwrap().grid[c])
  }
  fn get_mut(
    &mut self,
    c: DDD,
  ) -> Option<&mut Block> {
    if c.1 < 0 || c.1 > 255 {
      return None;
    }
    self
      .get_chunk_entity_or_queue(c)
      .map(move |entity| &mut self.chunks.get_mut(entity).unwrap().into_inner().grid[c])
  }
  fn get_many_mut<const N: usize>(&mut self, cs: [DDD; N]) -> Option<[&mut Block; N]> {
    for i in 0..N {
      for j in 0..i {
        if cs[i] == cs[j] {
          return None;
        }
      }
    }
    let mut chunk_entities: [Entity; N] = unsafe { MaybeUninit::uninit().assume_init() };
    for i in 0..N {
      let c = cs[i];
      if c.1 < 0 || c.1 > 255 {
        return None;
      }
      match self.get_chunk_entity_or_queue(c) {
        None => return None,
        Some(entity) => {
          chunk_entities[i] = entity;
        }
      }
    }
    Some(
      chunk_entities
        .map(|e| unsafe { self.chunks.get_unchecked(e).unwrap() })
        .into_iter()
        .enumerate()
        .map(|(i, c)| &mut c.into_inner().grid[cs[i]])
        .collect::<Vec<_>>()
        .try_into()
        .unwrap(),
    )
  }
}

pub struct ChunkMap {
  pub map: HashMap<DD, ChunkMeta>,
}

impl FromWorld for ChunkMap {
  fn from_world(_world: &mut World) -> Self {
    Self { map: HashMap::new() }
  }
}

#[derive(Component)]
pub struct ChunkTask {
  pub task: Task<(Chunk, DD)>,
}

pub enum LightPropagationType {
  Dim,
  Brighten
}

impl ChunkMap {
//   pub fn propagate_light<'a>(
//     &mut self,
//     chunks: &'a mut Query<&mut Chunk>,
//     idx: DDD,
//     propagate: LightPropagationType
//   ) {
//     // Heaven
//     // let atop = self.get_light_level(commands)
//     // Hearth
//   }
//   // pub fn get_light_level_mut<'a>(
//   //   &mut self,
//   //   chunks: &'a mut Query<&mut Chunk>,
//   //   idx: DDD,
//   // ) -> Option<&'a mut LightLevel> {
//   //   if let Some(mut chunk) = self.get_chunk_entity_or_queue(&None, idx).map(|entity| chunks.get_mut(entity).ok().map(|i|i.into_inner())).flatten() {
//   //     Some(&mut chunk.light_map[idx])
//   //   } else {
//   //     None
//   //   }
//   // }
//   // pub fn get_light_level(
//   //   &mut self,
//   //   chunks: &Query<&Chunk>,
//   //   idx: DDD,
//   // ) -> LightLevel {
//   //   if let Some(chunk) = self.get_chunk_entity_or_queue(&None, idx).map(|entity| chunks.get(entity).ok()).flatten() {
//   //     chunk.light_map[idx]
//   //   } else {
//   //     LightLevel::dark()
//   //   }
//   // }
//   // pub fn replace_light_level(
//   //   &mut self,
//   //   mut chunks: &mut Query<&mut Chunk>,
//   //   idx: DDD,
//   //   light_level: LightLevel
//   // ) -> LightLevel {
//   //   if let Some(mut chunk) = self.get_chunk_entity_or_queue(&None, idx).map(|entity| chunks.get_mut(entity).ok()).flatten() {
//   //     std::mem::replace(&mut chunk.light_map[idx], light_level)
//   //   } else {
//   //     LightLevel::dark()
//   //   }
//   // }
//   // pub fn set_light_level(
//   //   &mut self,
//   //   chunks: &mut Query<&mut Chunk>,
//   //   idx: DDD,
//   //   light_level: LightLevel
//   // ) {
//   //   if let Some(mut chunk) = self.get_chunk_entity_or_queue(&None, idx).map(|entity| chunks.get_mut(entity).ok()).flatten() {
//   //     chunk.light_map[idx] = light_level;
//   //   }
//   // }
//   pub fn get_chunk_entity_or_queue(
//     &mut self,
//     mut commands_dispatcher: CommandsDispatcher,
//     idx: DDD,
//   ) -> Option<Entity> {
//     let chunk_coord = ChunkMap::get_chunk_coord(idx);
//     match self.map.get(&chunk_coord) {
//       None => {
//         if let Some(mut commands_dispatcher) = commands_dispatcher.as_mut() {
//           let task = commands_dispatcher.dispatcher.spawn(Chunk::generate(chunk_coord));
//           self.map.insert(
//             chunk_coord,
//             ChunkMeta::new(commands_dispatcher.commands.spawn().insert(ChunkTask { task }).id()),
//           );
//         }
//         None
//       }
//       Some(ChunkMeta { generated, entity }) => {
//         if *generated {
//           Some(*entity)
//         } else {
//           None
//         }
//       }
//     }
//   }
//   pub fn get<'a>(
//     &mut self,
//     commands_dispatcher: CommandsDispatcher,
//     chunks: &'a Query<&Chunk>,
//     idx: DDD,
//   ) -> Option<&'a Block> {
//     if idx.1 < 0 || idx.1 > 255 {
//       return None;
//     }
//     self
//       .get_chunk_entity_or_queue(commands_dispatcher, idx)
//       .map(|entity| &chunks.get(entity).unwrap().grid[idx])
//   }
//   pub fn get_mut<'a>(
//     &mut self,
//     commands_dispatcher: CommandsDispatcher,
//     chunks: &'a mut Query<&mut Chunk>,
//     idx: DDD,
//   ) -> Option<&'a mut Block> {
//     if idx.1 < 0 || idx.1 > 255 {
//       return None;
//     }
//     self
//       .get_chunk_entity_or_queue(commands_dispatcher, idx)
//       .map(|entity| &mut chunks.get_mut(entity).unwrap().into_inner().grid[idx])
//   }
//   pub fn get_many_mut<'a, const N: usize>(
//     &mut self,
//     commands_dispatcher: CommandsDispatcher,
//     chunks: &'a mut Query<&mut Chunk>,
//     idxs: [DDD; N],
//   ) -> Option<[&'a mut Block; N]> {
//     for i in 0..N {
//       for j in 0..i {
//         if idxs[i] == idxs[j] {
//           return None;
//         }
//       }
//     }
//     let mut chunk_entities: [Entity; N] = unsafe { MaybeUninit::uninit().assume_init() };
//     for i in 0..N {
//       let idx = idxs[i];
//       if idx.1 < 0 || idx.1 > 255 {
//         return None;
//       }
//       match self.get_chunk_entity_or_queue(commands_dispatcher, idx) {
//         None => return None,
//         Some(entity) => {
//           chunk_entities[i] = entity;
//         }
//       }
//     }
//     Some(
//       chunk_entities
//         .map(|e| unsafe { chunks.get_unchecked(e).unwrap() })
//         .into_iter()
//         .enumerate()
//         .map(|(i, c)| &mut c.into_inner().grid[idxs[i]])
//         .collect::<Vec<_>>()
//         .try_into()
//         .unwrap(),
//     )
//   }
//   pub fn get_many_mut_with_free_entities<'a, const N: usize>(
//     &mut self,
//     commands_dispatcher: CommandsDispatcher,
//     chunks: &'a mut Query<&mut Chunk>,
//     idxs: [DDD; N],
//   ) -> Option<[(&'a mut Block, &'a mut BlockDataStorage); N]> {
//     for i in 0..N {
//       for j in 0..i {
//         if idxs[i] == idxs[j] {
//           return None;
//         }
//       }
//     }
//     let mut chunk_entities: [Entity; N] = unsafe { MaybeUninit::uninit().assume_init() };
//     for i in 0..N {
//       let idx = idxs[i];
//       if idx.1 < 0 || idx.1 > 255 {
//         return None;
//       }
//       match self.get_chunk_entity_or_queue(commands_dispatcher, idx) {
//         None => return None,
//         Some(entity) => {
//           chunk_entities[i] = entity;
//         }
//       }
//     }
//     Some(
//       chunk_entities
//         .map(|e| unsafe { chunks.get_unchecked(e).unwrap() })
//         .into_iter()
//         .enumerate()
//         .map(|(i, c)| {
//           let c = c.into_inner();
//           (&mut c.grid[idxs[i]], &mut c.sparse_storage)
//         })
//         .collect::<Vec<_>>()
//         .try_into()
//         .unwrap(),
//     )
//   }
//   pub fn animate(&mut self, source: DDD, target: DDD, commands: &mut Commands, chunks: &mut Query<&mut Chunk>, source_replace: BlockId) {
//     // let [(source_block, free_entities), (target_block, _)] = self.get_many_mut_with_free_entities(commands, None, chunks, [source, target]).unwrap();
//     // let block = std::mem::replace(source_block, Block::new(source_replace));
//     // let _ = std::mem::replace(target_block, Block::new(BlockId::Reserved));
//     // free_entities.push(
//     //   commands.spawn()
//     //     .insert(Transform::from_translation(from_ddd(source)))
//     //     .insert(Animation::new(source, target, 1.0, Some(block)))
//     //     .insert(RigidBody::KinematicPositionBased)
//     //     .with_children(|c| {
//     //       c.spawn()
//     //         .insert(Collider::cuboid(0.5, 0.5, 0.5))
//     //         .insert(Friction {
//     //           coefficient: 0.0,
//     //           combine_rule: CoefficientCombineRule::Min,
//     //         })
//     //         .insert(SolverGroups::new(0b10, 0b01))
//     //         .insert(CollisionGroups::new(0b10, 0b01))
//     //         .insert(Transform::from_translation(Vec3::new(0.5, 0.5, 0.5)))
//     //         .insert(GlobalTransform::default());
//     //     })
//     //     .id()
//     // );
//   }
  pub fn get_chunk_coord(mut coord: DDD) -> DD {
    if coord.0 < 0 {
      coord.0 -= 15;
    }
    if coord.2 < 0 {
      coord.2 -= 15;
    }
    (coord.0 / 16, coord.2 / 16)
  }
}