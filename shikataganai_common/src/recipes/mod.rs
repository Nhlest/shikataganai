use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::components::item::ItemId;
use crate::util::array::{Array, DDD};
use bevy::ecs::system::Resource;
use crate::recipes::in_world::populate_in_world_recipes;

pub mod in_world;

#[derive(Clone)]
pub struct SimpleRecipe {
  pub from: Array<DDD, BlockId>,
  pub to: Array<DDD, BlockId>,
  pub item: Option<ItemId>
}

#[derive(Resource)]
pub struct Recipes {
  pub recipes: Vec<SimpleRecipe>
}

impl Default for Recipes {
  fn default() -> Self {
    Self {
      recipes: populate_in_world_recipes().into_iter().collect()
    }
  }
}