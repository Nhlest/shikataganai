use crate::ecs::components::blocks::BlockTrait;

pub struct Air;
pub struct Dirt;
pub struct Grass;
pub struct Cobblestone;
pub struct Iron;
pub struct Furnace;

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

impl BlockTrait for Iron {}

impl BlockTrait for Furnace {}
