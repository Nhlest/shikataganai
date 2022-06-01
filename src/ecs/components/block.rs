use crate::ecs::resources::block::BlockSprite;
use bevy::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u16)]
pub enum BlockId {
  Air,
  Reserved,
  Dirt,
  Grass,
  Cobble,
  Hoist,
  Grid
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BlockMeta {
  v: u16
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BlockSparseId {
  id: u16
}

impl BlockId {
  pub fn into_array_of_faces(self) -> [BlockSprite; 6] {
    use crate::ecs::resources::block::BlockSprite::*;
    match self {
      BlockId::Air => [Empty, Empty, Empty, Empty, Empty, Empty],
      BlockId::Reserved => [Empty, Empty, Empty, Empty, Empty, Empty],
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
      BlockId::Grid => [Grid,Grid,Grid,Grid,Grid,Grid,],
    }
  }
}

#[derive(Debug, Component)]
pub struct Block {
  pub block: BlockId,
  pub meta: BlockMeta,
  pub sparse: BlockSparseId,
  pub reserved: u16
}

impl Block {
  pub fn new(block: BlockId) -> Self {
    Self {
      block,
      meta: BlockMeta { v: 0},
      sparse: BlockSparseId { id: 0},
      reserved: 0
    }
  }
  pub fn visible(&self) -> bool {
    match self.block {
      BlockId::Air | BlockId::Reserved => false,
      _ => true
    }
  }
  pub fn passable(&self) -> bool {
    match self.block {
      BlockId::Air | BlockId::Reserved => true,
      _ => false
    }
  }
}
