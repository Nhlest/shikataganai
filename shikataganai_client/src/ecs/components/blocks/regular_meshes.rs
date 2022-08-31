use crate::ecs::components::blocks::{BlockRenderInfo, BlockTraitExt};
use crate::ecs::plugins::rendering::mesh_pipeline::loader::Meshes;

pub struct Stair;

impl BlockTraitExt for Stair {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::AsMesh(Meshes::Stair)
  }
}
