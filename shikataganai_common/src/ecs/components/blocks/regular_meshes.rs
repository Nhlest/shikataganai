use crate::ecs::components::blocks::BlockTrait;

pub struct Stair;

impl BlockTrait for Stair {
  fn visible(&self) -> bool {
    false
  }
}
