use crate::ecs::components::block::{Block, BlockId};
use crate::ecs::components::chunk::Chunk;
use crate::util::array::{DD, DDD, from_ddd};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;
use std::mem::MaybeUninit;
use bevy_rapier3d::prelude::*;
use crate::ecs::plugins::animation::Animation;

pub struct ChunkMeta {
  pub entity: Entity,
  pub generated: bool,
}

impl ChunkMeta {
  pub fn new(entity: Entity) -> Self {
    Self {
      entity,
      generated: false,
    }
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


impl ChunkMap {
  pub fn get_chunk_entity_or_queue<'a>(
    &mut self,
    commands: &mut Commands,
    dispatcher: Option<&AsyncComputeTaskPool>,
    idx: DDD,
  ) -> Option<Entity> {
    let chunk_coord = self.get_chunk_coord(idx);
    match self.map.get(&chunk_coord) {
      None => {
        if let Some(dispatcher) = dispatcher {
          let task = dispatcher.spawn(Chunk::generate(chunk_coord));
          self.map.insert(
            chunk_coord,
            ChunkMeta::new(commands.spawn().insert(ChunkTask { task }).id()),
          );
        }
        None
      }
      Some(ChunkMeta { generated, entity }) => {
        if *generated {
          Some(*entity)
        } else {
          None
        }
      }
    }
  }
  pub fn get<'a>(
    &mut self,
    commands: &mut Commands,
    dispatcher: Option<&AsyncComputeTaskPool>,
    chunks: &'a Query<&Chunk>,
    idx: DDD,
  ) -> Option<&'a Block> {
    if idx.1 < 0 || idx.1 > 255 {
      return None;
    }
    self
      .get_chunk_entity_or_queue(commands, dispatcher, idx)
      .map(|entity| &chunks.get(entity).unwrap().grid[idx])
  }
  pub fn get_mut<'a>(
    &mut self,
    commands: &mut Commands,
    dispatcher: Option<&AsyncComputeTaskPool>,
    chunks: &'a mut Query<&mut Chunk>,
    idx: DDD,
  ) -> Option<&'a mut Block> {
    if idx.1 < 0 || idx.1 > 255 {
      return None;
    }
    self
      .get_chunk_entity_or_queue(commands, dispatcher, idx)
      .map(|entity| &mut chunks.get_mut(entity).unwrap().into_inner().grid[idx])
  }
  pub fn get_many_mut<'a, const N: usize>(
    &mut self,
    commands: &mut Commands,
    dispatcher: Option<&AsyncComputeTaskPool>,
    chunks: &'a mut Query<&mut Chunk>,
    idxs: [DDD; N],
  ) -> Option<[&'a mut Block; N]> {
    for i in 0..N {
      for j in 0..i {
        if idxs[i] == idxs[j] {
          return None;
        }
      }
    }
    let mut chunk_entities: [Entity; N] = unsafe { MaybeUninit::uninit().assume_init() };
    for i in 0..N {
      let idx = idxs[i];
      if idx.1 < 0 || idx.1 > 255 {
        return None;
      }
      match self.get_chunk_entity_or_queue(commands, dispatcher, idx) {
        None => return None,
        Some(entity) => {
          chunk_entities[i] = entity;
        }
      }
    }
    Some(
      chunk_entities
        .map(|e| unsafe { chunks.get_unchecked(e).unwrap() })
        .into_iter()
        .enumerate()
        .map(|(i, c)| &mut c.into_inner().grid[idxs[i]])
        .collect::<Vec<_>>()
        .try_into()
        .unwrap(),
    )
  }
  pub fn get_many_mut_with_free_entities<'a, const N: usize>(
    &mut self,
    commands: &mut Commands,
    dispatcher: Option<&AsyncComputeTaskPool>,
    chunks: &'a mut Query<&mut Chunk>,
    idxs: [DDD; N],
  ) -> Option<[(&'a mut Block, &'a mut Vec<Entity>); N]> {
    for i in 0..N {
      for j in 0..i {
        if idxs[i] == idxs[j] {
          return None;
        }
      }
    }
    let mut chunk_entities: [Entity; N] = unsafe { MaybeUninit::uninit().assume_init() };
    for i in 0..N {
      let idx = idxs[i];
      if idx.1 < 0 || idx.1 > 255 {
        return None;
      }
      match self.get_chunk_entity_or_queue(commands, dispatcher, idx) {
        None => return None,
        Some(entity) => {
          chunk_entities[i] = entity;
        }
      }
    }
    Some(
      chunk_entities
        .map(|e| unsafe { chunks.get_unchecked(e).unwrap() })
        .into_iter()
        .enumerate()
        .map(|(i, c)| {
          let c = c.into_inner();
          (&mut c.grid[idxs[i]], &mut c.free_entities)
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap(),
    )
  }
  pub fn animate(&mut self, source: DDD, target: DDD, commands: &mut Commands, chunks: &mut Query<&mut Chunk>, source_replace: BlockId) {
    let [(source_block, free_entities), (target_block, _)] = self.get_many_mut_with_free_entities(commands, None, chunks, [source, target]).unwrap();
    let block = std::mem::replace(source_block, Block::new(source_replace));
    let _ = std::mem::replace(target_block, Block::new(BlockId::Reserved));
    free_entities.push(
      commands.spawn()
        .insert(Transform::from_translation(from_ddd(source)))
        .insert(Animation::new(source, target, 1.0, Some(block)))
        .insert(RigidBody::KinematicPositionBased)
        .with_children(|c| {
          c.spawn()
            .insert(Collider::cuboid(0.5, 0.5, 0.5))
            .insert(Friction {
              coefficient: 0.0,
              combine_rule: CoefficientCombineRule::Min,
            })
            .insert(SolverGroups::new(0b10, 0b01))
            .insert(CollisionGroups::new(0b10, 0b01))
            .insert(Transform::from_translation(Vec3::new(0.5, 0.5, 0.5)))
            .insert(GlobalTransform::default());
        })
        .id()
    );
  }
  pub fn get_chunk_coord(&self, mut coord: DDD) -> DD {
    if coord.0 < 0 {
      coord.0 -= 16;
    }
    if coord.2 < 0 {
      coord.2 -= 16;
    }
    (coord.0 / 16, coord.2 / 16)
  }
}
