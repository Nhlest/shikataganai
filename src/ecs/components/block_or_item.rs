use crate::ecs::components::block::BlockId;
use crate::ecs::components::item::ItemId;
use bevy::ecs::component::Component;

#[derive(Component, Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum BlockOrItem {
  Block(BlockId),
  Item(ItemId),
}
