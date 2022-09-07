use crate::ecs::components::blocks::BlockTrait;

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
}

impl BlockTrait for Dirt {}

impl BlockTrait for Grass {}

impl BlockTrait for Cobblestone {}

impl BlockTrait for LightEmitter {
  fn visible(&self) -> bool {
    false
  }
  fn passable(&self) -> bool {
    true
  }
}
