use crate::ecs::components::blocks::BlockTrait;
use crate::ecs::components::functors::InternalInventory;
use crate::util::array::DDD;
use bevy::ecs::system::EntityCommands;

pub struct Stair;
pub struct Chest;

impl BlockTrait for Stair {
  fn visible(&self) -> bool {
    false
  }
}

impl BlockTrait for Chest {
  fn visible(&self) -> bool {
    false
  }

  fn need_to_spawn_functors(&self) -> bool {
    true
  }

  fn spawn_functors(&self, _location: DDD, commands: &mut EntityCommands) {
    commands.insert(InternalInventory::with_capacity(10));
  }

  fn need_reverse_location(&self) -> bool {
    true
  }
}
