use crate::ecs::plugins::rendering::mesh_pipeline::loader::Meshes;
use crate::ecs::resources::block::BlockSprite;
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::Block;

pub mod regular_blocks;
pub mod regular_meshes;

pub enum BlockRenderInfo {
  Nothing,
  AsBlock([BlockSprite; 6]),
  AsMesh(Meshes),
}

pub trait BlockTraitExt {
  fn render_info(&self) -> BlockRenderInfo;
}

pub trait DerefExt {
  fn deref_ext(&self) -> &dyn BlockTraitExt;
}

static BLOCK_TRAITS_EXT: [&(dyn BlockTraitExt + Sync); 6] = [
  &regular_blocks::Air,
  &regular_blocks::Dirt,
  &regular_blocks::Grass,
  &regular_blocks::Cobblestone,
  &regular_meshes::Stair,
  &regular_blocks::LightEmitter,
];

impl DerefExt for BlockId {
  #[inline]
  fn deref_ext(&self) -> &'static dyn BlockTraitExt {
    BLOCK_TRAITS_EXT[*self as usize]
  }
}

impl DerefExt for Block {
  #[inline]
  fn deref_ext(&self) -> &'static (dyn BlockTraitExt + '_) {
    self.block.deref_ext()
  }
}