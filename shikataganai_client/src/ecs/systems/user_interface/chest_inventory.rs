use bevy::prelude::Entity;

use crate::ecs::plugins::client::Requested;
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::plugins::rendering::inventory_pipeline::InventoryTextureOutputHandle;
use crate::ecs::resources::player::PlayerInventory;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_renet::renet::RenetClient;
use bincode::serialize;
use egui::{emath, Color32, Id, TextStyle, Widget};
use shikataganai_common::ecs::components::blocks::{QuantifiedBlockOrItem, ReverseLocation};
use shikataganai_common::ecs::components::functors::InternalInventory;
use shikataganai_common::networking::{ClientChannel, FunctorType, PlayerCommand};

#[derive(Resource)]
pub struct InventoryOpened(pub Entity);

#[derive(Default, Resource)]
pub enum InventoryItemMovementStatus {
  #[default]
  Nothing,
  HoldingItemFrom(usize),
}

pub fn chest_inventory(
  mut commands: Commands,
  mut egui: ResMut<EguiContext>,
  window: Res<Windows>,
  inventory_opened: Option<ResMut<InventoryOpened>>,
  inventory_query: Query<&mut InternalInventory>,
  requested_query: Query<&Requested>,
  location_query: Query<&ReverseLocation>,
  mut client: ResMut<RenetClient>,
  inventory_movement: ResMut<InventoryItemMovementStatus>,
  mut extracted_items: ResMut<ExtractedItems>,
  inventory_texture: Res<InventoryTextureOutputHandle>,
  player_inventory: Res<PlayerInventory>,
) {
  let active_window = window.get_primary().unwrap();
  if let Some(inventory_entity) = inventory_opened.map(|e| e.0) {
    match inventory_query.get(inventory_entity) {
      Ok(internal_inventory) => {
        let ui = egui.ctx_mut();

        egui::Window::new("Inventory")
          .title_bar(false)
          .resizable(false)
          .fixed_pos(
            [
              active_window.width() / 2.0 - 1080.0 / 2.0,
              active_window.height() - 600.0,
            ],
          )
          .fixed_size([1080.0, 600.0])
          .show(ui, |ui| {
            if let InventoryItemMovementStatus::HoldingItemFrom(from_slot) = *inventory_movement {
              egui::popup::show_tooltip(ui.ctx(), Id::from("Tooltip"), |ui| {
                let block_or_item = internal_inventory.inventory.get(from_slot).unwrap().as_ref().unwrap().block_or_item;
                let coords = extracted_items.request(block_or_item).unwrap_or((0.0, 0.0));
                egui::Image::new(inventory_texture.1, [95.0, 95.0]).uv([[coords.0, coords.1].into(), [coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0].into()]).ui(ui);
              });
            }
            ui.style_mut().spacing.button_padding = emath::Vec2::ZERO;
            // let mut swap = None;
            egui::Grid::new("Chest Inventory Grid").show(ui, |ui| {
              for (i, item) in internal_inventory.inventory.iter().enumerate() {
                if let InventoryItemMovementStatus::HoldingItemFrom(from_slot) = *inventory_movement && from_slot == i {
                  egui::ImageButton::new(inventory_texture.1, [95.0, 95.0]).uv([[1.0, 1.1].into(), [1.0, 1.0].into()]).ui(ui);
                } else {
                  if match item {
                    None => {
                      egui::ImageButton::new(inventory_texture.1, [95.0, 95.0]).uv([[1.0, 1.1].into(), [1.0, 1.0].into()]).ui(ui)
                    }
                    Some(QuantifiedBlockOrItem { block_or_item, quant }) => {
                      ui.allocate_ui([95.0, 95.0].into(), |ui| {
                        let text = egui::WidgetText::RichText(egui::RichText::new(format!("{}", quant))).into_galley(ui, None, 50.0, TextStyle::Button);
                        let coords = extracted_items.request(*block_or_item).unwrap_or((0.0, 0.0));
                        let pos = ui.next_widget_position();
                        let sense = egui::ImageButton::new(inventory_texture.1, [95.0, 95.0]).uv([[coords.0, coords.1].into(), [coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0].into()]).ui(ui);
                        text.paint_with_fallback_color(ui.painter(), pos, Color32::WHITE);
                        sense
                      }).inner
                    }
                  }.clicked() {
                    // match *item_move {
                    //   ItemMove::Nothing => {
                    //     if item.is_some() {
                    //       *item_move = ItemMove::FromSlot(i);
                    //     }
                    //   }
                    //   ItemMove::FromSlot(from_slot) => {
                    //     swap = Some((from_slot, i));
                    //   }
                    // }
                  }
                }
                if i % player_inventory.hot_bar_width == player_inventory.hot_bar_width - 1 {
                  ui.end_row();
                }
              }
            });
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
