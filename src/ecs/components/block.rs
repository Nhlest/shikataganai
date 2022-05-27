use crate::ecs::resources::block::BlockSprite;
use bevy::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BlockId {
  Air,
  Dirt,
  Grass,
  Cobble,
  Hoist,
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

#[derive(Debug, Component)]
pub struct Block {
  pub block: BlockId,
}
