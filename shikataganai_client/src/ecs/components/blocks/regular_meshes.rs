use crate::ecs::components::blocks::{BlockRenderInfo, BlockTraitExt, Skeletons};
use crate::ecs::plugins::rendering::mesh_pipeline::loader::Meshes;

pub struct Stair;
pub struct Chest;

impl BlockTraitExt for Stair {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::AsMesh(Meshes::Stair)
  }
}

impl BlockTraitExt for Chest {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::AsSkeleton(Skeletons::Chest)
  }
}
