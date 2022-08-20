use crate::ecs::components::blocks::{BlockRenderInfo, BlockTrait};

pub struct Air;
pub struct Dirt;
pub struct Grass;
pub struct Cobblestone;
pub struct LightEmitter;

impl BlockTrait for Air {
  fn visible(&self) -> bool {
    false
  }
  fn passable(&self) -> bool {
    true
  }
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::Nothing
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

impl BlockTrait for LightEmitter {
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
