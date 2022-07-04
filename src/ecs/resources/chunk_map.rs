use crate::ecs::components::block::Block;
use crate::ecs::components::chunk::Chunk;
use crate::ecs::resources::light::LightLevel;
use crate::util::array::{ImmediateNeighbours, DD, DDD};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::utils::HashMap;
use duplicate::duplicate_item;
use std::mem::MaybeUninit;

pub struct ChunkMeta {
  pub entity: Entity,
}

impl ChunkMeta {
  pub fn new(entity: Entity) -> Self {
    Self { entity }
  }
}

#[derive(SystemParam)]
pub struct BlockAccessorSpawner<'w, 's> {
  pub chunk_map: ResMut<'w, ChunkMap>,
  pub chunks: Query<'w, 's, &'static mut Chunk>,
  pub commands: Commands<'w, 's>,
  pub dispatcher: Res<'w, AsyncComputeTaskPool>,
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
      None => None,
      Some(ChunkMeta { entity }) => Some(*entity),
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
      Some(ChunkMeta { entity }) => Some(*entity),
    }
  }
}

pub trait BlockAccessor {
  fn get_single(&mut self, c: DDD) -> Option<&Block>;
  fn get_mut(&mut self, c: DDD) -> Option<&mut Block>;
  fn get_many_mut<const N: usize>(&mut self, cs: [DDD; N]) -> Option<[&mut Block; N]>;
  fn get_light_level(&mut self, c: DDD) -> Option<LightLevel>;
  fn set_light_level(&mut self, c: DDD, light: LightLevel);
  fn propagate_light(&mut self, c: DDD);
}

#[duplicate_item(T; [BlockAccessorSpawner]; [BlockAccessorStatic])]
// impl<'w, 's> BlockAccessor for BlockAccessorSpawner<'w, 's> {
impl<'w, 's> BlockAccessor for T<'w, 's> {
  fn get_single(&mut self, c: DDD) -> Option<&Block> {
    if c.1 < 0 || c.1 > 255 {
      return None;
    }
    self
      .get_chunk_entity_or_queue(c)
      .map(move |entity| &self.chunks.get(entity).unwrap().grid[c])
  }
  fn get_mut(&mut self, c: DDD) -> Option<&mut Block> {
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
  fn get_light_level(&mut self, c: DDD) -> Option<LightLevel> {
    self
      .get_chunk_entity_or_queue(c)
      .map(|entity| {
        let chunk = self.chunks.get(entity).unwrap();
        (!chunk.grid[c].visible()).then_some(chunk.light_map[c])
      })
      .flatten()
  }
  fn set_light_level(&mut self, c: DDD, light: LightLevel) {
    self.get_chunk_entity_or_queue(c).map(|entity| {
      let mut chunk = self.chunks.get_mut(entity).unwrap();
      if !chunk.grid[c].visible() {
        chunk.light_map[c] = light;
      }
    });
  }
  fn propagate_light(&mut self, c: DDD) {
    if let Some(current_light) = self.get_light_level(c) {
      let mut new_heaven_light = None;
      let mut new_hearth_light = None;
      for heaven_check in c.immeidate_neighbours() {
        if let Some(LightLevel { mut heaven, hearth }) = self.get_light_level(heaven_check) {
          if heaven_check.1 - c.1 == 1 {
            heaven += 1
          }
          if current_light.heaven < heaven - 1 {
            new_heaven_light = Some(heaven - 1);
          }
          if current_light.hearth < hearth - 1 {
            new_hearth_light = Some(hearth - 1);
          }
        }
      }
      if new_heaven_light.is_none() && new_hearth_light.is_none() {
        return;
      }
      self.set_light_level(
        c,
        LightLevel::new(
          new_heaven_light.unwrap_or(current_light.heaven),
          new_hearth_light.unwrap_or(current_light.hearth),
        ),
      );
      for i in c.immeidate_neighbours() {
        self.propagate_light(i);
      }
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
