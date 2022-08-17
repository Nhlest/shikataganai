use crate::ecs::components::blocks::BlockTrait;
use std::ops::Deref;

#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
#[repr(u32)]
pub enum BlockId {
  Air,
  Dirt,
  Grass,
  Cobble,
  Stair,
}

impl Deref for BlockId {
  type Target = dyn BlockTrait;

  fn deref(&self) -> &'static Self::Target {
    use crate::ecs::components::blocks::*;
    match self {
      BlockId::Air => &regular_blocks::Air,
      BlockId::Dirt => &regular_blocks::Dirt,
      BlockId::Grass => &regular_blocks::Grass,
      BlockId::Cobble => &regular_blocks::Cobblestone,
      BlockId::Stair => &regular_meshes::Stair,
    }
  }
}
