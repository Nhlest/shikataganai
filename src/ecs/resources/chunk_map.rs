use std::mem::MaybeUninit;
use crate::ecs::components::block::{Block};
use crate::ecs::components::chunk::Chunk;
use crate::util::array::{DD, DDD};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;

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

#[derive(Clone)]
pub struct ChunkMapSize {
  pub x: i32,
  pub y: i32,
}

impl Default for ChunkMapSize {
  fn default() -> Self {
    Self { x: 5, y: 5 }
  }
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
  pub fn get<'a>(&mut self, commands: &mut Commands, dispatcher: &AsyncComputeTaskPool, chunks: &'a Query<&Chunk>, idx: DDD) -> Option<&'a Block> {
    if idx.1 < 0 || idx.1 > 255 {
      return None;
    }
    let chunk_coord = self.get_chunk_coord(idx);
    match self.map.get(&chunk_coord) {
      None => {
        let task = dispatcher.spawn(Chunk::generate(chunk_coord));
        self.map.insert(
          chunk_coord,
          ChunkMeta::new(commands.spawn().insert(ChunkTask { task }).id()),
        );
        None
      }
      Some(ChunkMeta { generated, entity }) => {
        if *generated {
          Some(&chunks.get(*entity).unwrap().grid[idx])
        } else {
          None
        }
      }
    }
  }
  pub fn get_mut<'a>(&mut self, commands: &mut Commands, dispatcher: &AsyncComputeTaskPool, chunks: &'a mut Query<&mut Chunk>, idx: DDD) -> Option<&'a mut Block> {
    if idx.1 < 0 || idx.1 > 255 {
      return None;
    }
    let chunk_coord = self.get_chunk_coord(idx);
    match self.map.get(&chunk_coord) {
      None => {
        let task = dispatcher.spawn(Chunk::generate(chunk_coord));
        self.map.insert(
          chunk_coord,
          ChunkMeta::new(commands.spawn().insert(ChunkTask { task }).id()),
        );
        None
      }
      Some(ChunkMeta { generated, entity }) => {
        if *generated {
          Some(&mut chunks.get_mut(*entity).unwrap().into_inner().grid[idx])
        } else {
          None
        }
      }
    }
  }
  pub fn get_many_mut<'a, const N: usize>(&mut self, commands: &mut Commands, dispatcher: &AsyncComputeTaskPool, chunks: &'a mut Query<&mut Chunk>, idxs: [DDD; N]) -> Option<[&'a mut Block; N]> {
    for i in 0..N {
      for j in 0..i {
        if idxs[i] == idxs[j] { return None; }
      }
    }
    let mut chunk_entities : [Entity; N] = unsafe { MaybeUninit::uninit().assume_init() };
    for i in 0..N {
      let idx = idxs[i];
      if idx.1 < 0 || idx.1 > 255 {
        return None;
      }
      let chunk_coord = self.get_chunk_coord(idx);
      match self.map.get(&chunk_coord) {
        None => {
          let task = dispatcher.spawn(Chunk::generate(chunk_coord));
          self.map.insert(
            chunk_coord,
            ChunkMeta::new(commands.spawn().insert(ChunkTask { task }).id()),
          );
          return None;
        }
        Some(ChunkMeta { generated, entity }) => {
          if *generated {
            chunk_entities[i] = *entity;
          } else {
            return None;
          }
        }
      }
    }
    Some(
      chunk_entities.map(|e| unsafe { chunks.get_unchecked(e).unwrap() })
        .into_iter()
        .enumerate()
        .map(|(i,c)|&mut c.into_inner().grid[idxs[i]])
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
    )
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
