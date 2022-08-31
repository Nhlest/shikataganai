use crate::ecs::components::blocks::{BlockRenderInfo, BlockTraitExt};

pub struct Air;
pub struct Dirt;
pub struct Grass;
pub struct Cobblestone;
pub struct LightEmitter;

impl BlockTraitExt for Air {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::Nothing
  }
}

impl BlockTraitExt for Dirt {
  fn render_info(&self) -> BlockRenderInfo {
    use crate::ecs::resources::block::BlockSprite::*;
    BlockRenderInfo::AsBlock([Dirt, Dirt, Dirt, Dirt, Dirt, Dirt])
  }
}

impl BlockTraitExt for Grass {
  fn render_info(&self) -> BlockRenderInfo {
    use crate::ecs::resources::block::BlockSprite::*;
    BlockRenderInfo::AsBlock([HalfGrass, HalfGrass, HalfGrass, HalfGrass, Grass, Dirt])
  }
}

impl BlockTraitExt for Cobblestone {
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

impl BlockTraitExt for LightEmitter {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::Nothing
  }
}
