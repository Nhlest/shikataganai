use crate::ecs::components::blocks::BlockTrait;

pub struct Stair;

impl BlockTrait for Stair {
  fn visible(&self) -> bool {
    false
  }
  // fn render_info(&self) -> BlockRenderInfo {
  //   BlockRenderInfo::AsMesh(Meshes::Stair)
  // }
}
