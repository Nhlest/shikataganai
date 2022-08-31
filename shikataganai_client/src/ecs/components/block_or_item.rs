use crate::ecs::components::item::ItemId;
use bevy::ecs::component::Component;
use shikataganai_common::ecs::components::blocks::block_id::BlockId;

#[derive(Component, Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum BlockOrItem {
  Block(BlockId),
  Item(ItemId),
}
