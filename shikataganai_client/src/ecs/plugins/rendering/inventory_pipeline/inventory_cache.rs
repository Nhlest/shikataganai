use bevy::prelude::Deref;
use bevy::prelude::Resource;
use bevy::utils::hashbrown::{HashMap, HashSet};
use shikataganai_common::ecs::components::blocks::BlockOrItem;

#[derive(Default, Resource)]
pub struct ExtractedItems {
  pub rendered: ItemRenderMap,
  pub requested: HashSet<BlockOrItem>,
  pub rerender: bool,
}

impl ExtractedItems {
  pub fn request(&mut self, block_or_item: BlockOrItem) -> Option<(f32, f32)> {
    let coord = self
      .rendered
      .0
      .get_mut(&block_or_item)
      .map(
        |ItemRenderEntry {
           coord,
           has_been_requested,
         }| {
          *has_been_requested = true;
          coord
        },
      )
      .copied();
    if coord.is_none() {
      self.requested.insert(block_or_item);
    }
    coord
  }

  pub fn clear(&mut self) {
    self.requested.clear();
  }
}

#[derive(Default, Debug, Clone)]
pub struct ItemRenderEntry {
  pub coord: (f32, f32),
  pub has_been_requested: bool,
}

#[derive(Deref, Default, Clone, Resource)]
pub struct ItemRenderMap(pub HashMap<BlockOrItem, ItemRenderEntry>);
