use crate::ecs::components::blocks::{regular_blocks, regular_meshes, BlockTrait, Block, BlockMeta};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use bevy::prelude::Entity;

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum BlockId {
  Air,
  Dirt,
  Grass,
  Cobble,
  Stair,
  LightEmitter,
}

impl Into<Block> for BlockId {
  fn into(self) -> Block {
    Block {
      block: self,
      meta: BlockMeta { v: 0 },
      entity: Entity::from_bits(0)
    }
  }
}

static BLOCK_TRAITS: [&(dyn BlockTrait + Sync); 6] = [
  &regular_blocks::Air,
  &regular_blocks::Dirt,
  &regular_blocks::Grass,
  &regular_blocks::Cobblestone,
  &regular_meshes::Stair,
  &regular_blocks::LightEmitter,
];

impl Deref for BlockId {
  type Target = dyn BlockTrait;

  #[inline]
  fn deref(&self) -> &'static Self::Target {
    BLOCK_TRAITS[*self as usize]
  }
}
