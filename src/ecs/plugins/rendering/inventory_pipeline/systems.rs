use crate::ecs::plugins::rendering::inventory_pipeline::{ExtractedItems, INVENTORY_OUTPUT_TEXTURE_WIDTH};
use crate::ecs::resources::player::{PlayerInventory, QuantifiedBlockOrItem, RerenderInventory};
use bevy::prelude::*;
use bevy::render::Extract;

pub fn prepare_extracted_inventory(
  player_inventory: Res<PlayerInventory>,
  mut extracted_items: ResMut<ExtractedItems>,
  rerender_inventory: Res<RerenderInventory>,
) {
  if rerender_inventory.0 {
    extracted_items.0.clear();

    let mut x = 0.0;
    let mut y = 0.0;

    for QuantifiedBlockOrItem { block_or_item, .. } in player_inventory.items.iter().filter_map(|x| x.as_ref()) {
      extracted_items.0.insert(block_or_item.clone(), (x, y));
      x += 1.0 / INVENTORY_OUTPUT_TEXTURE_WIDTH;
      if x >= 1.0 {
        y += 1.0 / INVENTORY_OUTPUT_TEXTURE_WIDTH;
        x = 0.0;
      }
    }
  }
}

pub fn extract_inventory_tiles(
  mut commands: Commands,
  rerender_inventory: Extract<Res<RerenderInventory>>,
  extracted_items: Extract<Res<ExtractedItems>>,
) {
  commands.insert_resource(RerenderInventory(rerender_inventory.0));
  if rerender_inventory.0 {
    commands.insert_resource(ExtractedItems(extracted_items.0.clone()));
  }
}

pub fn cleanup_rerender(mut rerender_inventory: ResMut<RerenderInventory>) {
  rerender_inventory.0 = false;
}
