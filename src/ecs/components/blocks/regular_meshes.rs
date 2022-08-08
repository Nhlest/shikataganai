use crate::ecs::components::blocks::{BlockRenderInfo, BlockTrait};
use crate::ecs::plugins::rendering::mesh_pipeline::loader::Meshes;

pub struct Stair;

impl BlockTrait for Stair {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::AsMesh(Meshes::Stair)
  }
}
