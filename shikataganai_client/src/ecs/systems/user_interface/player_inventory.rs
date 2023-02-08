use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::plugins::rendering::inventory_pipeline::InventoryTextureOutputHandle;
use crate::ecs::resources::player::PlayerInventory;
use crate::ecs::systems::user_interface::{InventoryItemMovementStatus, item_button_grid};
use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{emath, Context, Id, Ui, Widget};
use shikataganai_common::ecs::components::blocks::QuantifiedBlockOrItem;

#[derive(Resource)]
pub struct PlayerInventoryOpened;

pub fn render_player_inventory(
  ui: &mut Ui,
  player_inventory: &PlayerInventory,
  extracted_items: &mut ExtractedItems,
  inventory_texture: &InventoryTextureOutputHandle,
  item_move: &InventoryItemMovementStatus,
  range_start: usize
) -> Option<usize> {
  let content_fetch = |x| {
    if let InventoryItemMovementStatus::HoldingItemFrom(from_slot) = *item_move && from_slot == x {
      None
    } else {
      (player_inventory.items.get(x-range_start).unwrap() as &Option<QuantifiedBlockOrItem>).as_ref()
    }
  };
  let mut clicked = item_button_grid(
    "Top Grid",
    ui,
    content_fetch,
    range_start+player_inventory.hot_bar_width..range_start+player_inventory.items.len(),
    player_inventory.hot_bar_width,
    extracted_items,
    inventory_texture,
  );
  ui.separator();
  clicked = clicked.or(item_button_grid(
    "Bottom Grid",
    ui,
    content_fetch,
    range_start..range_start+player_inventory.hot_bar_width,
    player_inventory.hot_bar_width,
    extracted_items,
    inventory_texture,
  ));
  clicked
}

pub fn player_inventory(
  mut egui: ResMut<EguiContext>,
  window: Res<Windows>,
  inventory_opened: Option<ResMut<PlayerInventoryOpened>>,
  mut player_inventory: ResMut<PlayerInventory>,
  mut extracted_items: ResMut<ExtractedItems>,
  inventory_texture: Res<InventoryTextureOutputHandle>,
  mut item_move: ResMut<InventoryItemMovementStatus>,
) {
  if let Some(_) = inventory_opened {
    let active_window = window.get_primary().unwrap();
    let ui = egui.ctx_mut();
    egui::Window::new("Inventory")
      .title_bar(false)
      .resizable(false)
      .fixed_pos([
        active_window.width() / 2.0 - 1080.0 / 2.0,
        active_window.height() - 600.0,
      ])
      .fixed_size([1080.0, 600.0])
      .show(ui, |ui| {
        if let InventoryItemMovementStatus::HoldingItemFrom(from_slot) = *item_move {
          egui::popup::show_tooltip(ui.ctx(), Id::from("Tooltip"), |ui| {
            let block_or_item = player_inventory
              .items
              .get(from_slot)
              .unwrap()
              .as_ref()
              .unwrap()
              .block_or_item;
            let coords = extracted_items.request(block_or_item).unwrap_or((0.0, 0.0));
            egui::Image::new(inventory_texture.1, [95.0, 95.0])
              .uv([
                [coords.0, coords.1].into(),
                [coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0].into(),
              ])
              .ui(ui);
          });
        }
        ui.style_mut().spacing.button_padding = emath::Vec2::ZERO;
        let mut swap = None;
        let clicked = render_player_inventory(
          ui,
          player_inventory.as_ref(),
          extracted_items.as_mut(),
          inventory_texture.as_ref(),
          item_move.as_ref(),
          0
        );
        if let Some(clicked) = clicked {
          match *item_move {
            InventoryItemMovementStatus::Nothing => {
              if player_inventory.items.get(clicked).unwrap().is_some() {
                *item_move = InventoryItemMovementStatus::HoldingItemFrom(clicked);
              }
            }
            InventoryItemMovementStatus::HoldingItemFrom(from_slot) => {
              swap = Some((from_slot, clicked));
            }
          }
        }
        if let Some((from, to)) = swap {
          *item_move = InventoryItemMovementStatus::Nothing;
          player_inventory.items.swap(from, to);
        }
      });
  }
}
