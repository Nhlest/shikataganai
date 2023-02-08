use bevy::prelude::Entity;

use crate::ecs::plugins::client::Requested;
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::plugins::rendering::inventory_pipeline::InventoryTextureOutputHandle;
use crate::ecs::resources::player::PlayerInventory;
use crate::ecs::systems::user_interface::{InventoryItemMovementStatus, InventoryOpened, item_button_grid};
use crate::ecs::systems::user_interface::player_inventory::render_player_inventory;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_renet::renet::RenetClient;
use bincode::serialize;
use egui::{emath, Color32, Id, TextStyle, Widget};
use shikataganai_common::ecs::components::blocks::{QuantifiedBlockOrItem, ReverseLocation};
use shikataganai_common::ecs::components::functors::InternalInventory;
use shikataganai_common::networking::{ClientChannel, FunctorType, InventoryIndex, PlayerCommand};

pub fn chest_inventory(
  mut commands: Commands,
  mut egui: ResMut<EguiContext>,
  window: Res<Windows>,
  inventory_opened: Option<ResMut<InventoryOpened>>,
  mut inventory_query: Query<&mut InternalInventory>,
  requested_query: Query<&Requested>,
  location_query: Query<&ReverseLocation>,
  mut client: ResMut<RenetClient>,
  mut item_move: ResMut<InventoryItemMovementStatus>,
  mut extracted_items: ResMut<ExtractedItems>,
  inventory_texture: Res<InventoryTextureOutputHandle>,
  mut player_inventory: ResMut<PlayerInventory>,
) {
  let active_window = window.get_primary().unwrap();
  if let Some(inventory_entity) = inventory_opened.map(|e| e.0) {
    match inventory_query.get_mut(inventory_entity) {
      Ok(mut internal_inventory) => {
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
                let block_or_item = match InventoryIndex::from(from_slot) {
                  InventoryIndex::Local(from_slot) => {
                    internal_inventory
                      .inventory
                      .get(from_slot)
                      .unwrap()
                      .as_ref()
                      .unwrap()
                      .block_or_item
                  }
                  InventoryIndex::Foreign(from_slot) => {
                    player_inventory
                      .items
                      .get(from_slot)
                      .unwrap()
                      .as_ref()
                      .unwrap()
                      .block_or_item
                  }
                };
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

            let clicked = item_button_grid(
              "Chest Inventory",
              ui,
              |a| {
                if let InventoryItemMovementStatus::HoldingItemFrom(from_slot) = *item_move && from_slot == a {
                  None
                } else {
                  (internal_inventory.inventory.get(a).unwrap() as &Option<QuantifiedBlockOrItem>).as_ref()
                }
              },
              0..internal_inventory.inventory.len(),
              player_inventory.hot_bar_width,
              extracted_items.as_mut(),
              inventory_texture.as_ref(),
            );
            let clicked = clicked.or(render_player_inventory(
              ui,
              player_inventory.as_ref(),
              extracted_items.as_mut(),
              inventory_texture.as_ref(),
              item_move.as_ref(),
              1000
            ));
            let mut swap = None;
            if let Some(clicked) = clicked {
              match *item_move {
                InventoryItemMovementStatus::Nothing => {
                  if clicked < 1000 {
                    if internal_inventory.inventory.get(clicked).unwrap().is_some() {
                      *item_move = InventoryItemMovementStatus::HoldingItemFrom(clicked);
                    }
                  } else {
                    if player_inventory.items.get(clicked-1000).unwrap().is_some() {
                      *item_move = InventoryItemMovementStatus::HoldingItemFrom(clicked);
                    }
                  }
                }
                InventoryItemMovementStatus::HoldingItemFrom(from_slot) => {
                  swap = Some((from_slot, clicked));
                }
              }
            }
            if let Some((from, to)) = swap {
              *item_move = InventoryItemMovementStatus::Nothing;
              if from < 1000 && to < 1000 {
                internal_inventory.inventory.swap(from, to);
              } else if from >= 1000 && to >= 1000 {
                player_inventory.items.swap(from-1000, to-1000);
              } else if from < 1000 && to >= 1000 {
                let item = internal_inventory.inventory.get_mut(from).unwrap();
                let item2 = player_inventory.items.get_mut(to - 1000).unwrap();
                std::mem::swap(item, item2);
              } else {
                let item = internal_inventory.inventory.get_mut(to).unwrap();
                let item2 = player_inventory.items.get_mut(from - 1000).unwrap();
                std::mem::swap(item, item2);
              }
            }
          });
      }
      Err(_) => {
        if !requested_query.get(inventory_entity).is_ok() {
          let location = location_query.get(inventory_entity).unwrap();
          client.send_message(
            ClientChannel::ClientCommand.id(),
            serialize(&PlayerCommand::RequestFunctor {
              location: location.0,
              functor: FunctorType::InternalInventory,
            })
            .unwrap(),
          );
          commands.entity(inventory_entity).insert(Requested);
        }
      }
    }
  }
}
