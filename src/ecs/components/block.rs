use crate::ecs::resources::block::BlockSprite;
use bevy::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u32)]
pub enum BlockId {
  Air,
  Dirt,
  Grass,
  Cobble,
  Hoist,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BlockMeta {
  v: u32,
}

impl BlockId {
  pub fn into_array_of_faces(self) -> [BlockSprite; 6] {
    use crate::ecs::resources::block::BlockSprite::*;
    match self {
      BlockId::Air => [Empty, Empty, Empty, Empty, Empty, Empty],
      BlockId::Dirt => [Dirt, Dirt, Dirt, Dirt, Dirt, Dirt],
      BlockId::Grass => [HalfGrass, HalfGrass, HalfGrass, HalfGrass, Grass, Dirt],
      BlockId::Cobble => [
        Cobblestone,
        Cobblestone,
        Cobblestone,
        Cobblestone,
        Cobblestone,
        Cobblestone,
      ],
      BlockId::Hoist => [Wood, Wood, Wood, Wood, Wood, Wood],
    }
  }
}

#[derive(Debug, Component, Copy, Clone)]
pub struct Block {
  pub block: BlockId,
  pub meta: BlockMeta,
  pub entity: Entity,
}

impl Block {
  pub fn new(block: BlockId) -> Self {
    Self {
      block,
      meta: BlockMeta { v: 0 },
      entity: Entity::from_bits(0),
    }
  }
  pub fn visible(&self) -> bool {
    match self.block {
      BlockId::Air => false,
      _ => true,
    }
  }
  pub fn passable(&self) -> bool {
    match self.block {
      BlockId::Air => true,
      _ => false,
    }
  }
}
