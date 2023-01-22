use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::components::item::ItemId;
use crate::util::array::{Array, DDD};

pub mod in_world;

pub struct SimpleRecipe {
  from: Array<DDD, BlockId>,
  to: Array<DDD, BlockId>,
  item: Option<ItemId>
}