use bevy::prelude::{Commands, Entity};
use crate::ecs::components::blocks::{Block, BlockTrait};
use crate::util::array::DDD;

pub struct Stair;
pub struct Chest;

impl BlockTrait for Stair {
  fn visible(&self) -> bool {
    false
  }
  fn spawn_functors(&self, entity: Entity, location: DDD, commands: &mut Commands) {

  }
}

impl BlockTrait for Chest {
  fn visible(&self) -> bool {
    false
  }

  fn spawn_functors(&self, entity: Entity, location: DDD, commands: &mut Commands) {

  }
}