use crate::ecs::components::blocks::{BlockRenderInfo, BlockTrait};

pub struct Air;
pub struct Dirt;
pub struct Grass;
pub struct Cobblestone;

impl BlockTrait for Air {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::Nothing
  }
  fn visible(&self) -> bool {
    false
  }
  fn passable(&self) -> bool {
    true
  }
}

impl BlockTrait for Dirt {
  fn render_info(&self) -> BlockRenderInfo {
    use crate::ecs::resources::block::BlockSprite::*;
    BlockRenderInfo::AsBlock([Dirt, Dirt, Dirt, Dirt, Dirt, Dirt])
  }
}

impl BlockTrait for Grass {
  fn render_info(&self) -> BlockRenderInfo {
    use crate::ecs::resources::block::BlockSprite::*;
    BlockRenderInfo::AsBlock([HalfGrass, HalfGrass, HalfGrass, HalfGrass, Grass, Dirt])
  }
}

impl BlockTrait for Cobblestone {
  fn render_info(&self) -> BlockRenderInfo {
    use crate::ecs::resources::block::BlockSprite::*;
    BlockRenderInfo::AsBlock([
      Cobblestone,
      Cobblestone,
      Cobblestone,
      Cobblestone,
      Cobblestone,
      Cobblestone,
    ])
  }
}
