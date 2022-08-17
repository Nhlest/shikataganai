use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::components::item::ItemId;
use bevy::ecs::component::Component;

#[derive(Component, Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum BlockOrItem {
  Block(BlockId),
  Item(ItemId),
}
