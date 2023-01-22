use bevy::ecs::schedule::IntoRunCriteria;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Align, Color32, emath, Layout, Sense, TextStyle, Widget, Id};
use itertools::Itertools;
use shikataganai_common::ecs::components::blocks::{BlockOrItem, QuantifiedBlockOrItem};
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::plugins::rendering::inventory_pipeline::InventoryTextureOutputHandle;
use crate::ecs::resources::player::PlayerInventory;

#[derive(Resource)]
pub struct PlayerInventoryOpened;

#[derive(Resource, Default)]
pub enum ItemMove {
  #[default]
  Nothing,
  FromSlot(usize),
}

pub fn player_inventory(
  mut commands: Commands,
  mut egui: ResMut<EguiContext>,
  window: Res<Windows>,
  inventory_opened: Option<ResMut<PlayerInventoryOpened>>,
  mut player_inventory: ResMut<PlayerInventory>,
  // texture: Res<GUITextureAtlas>,
  mut extracted_items: ResMut<ExtractedItems>,
  inventory_texture: Res<InventoryTextureOutputHandle>,
  mut item_move: ResMut<ItemMove>
) {
  if let Some(_) = inventory_opened {
    let ui = egui.ctx_mut();
    let active_window = window.get_primary().unwrap();
    let ui = egui.ctx_mut();
    let x1 = active_window.width() / 2.0 - 2.0;
    let y1 = active_window.height() / 2.0 - 2.0;
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
        if let ItemMove::FromSlot(from_slot) = *item_move {
          egui::popup::show_tooltip(ui.ctx(), Id::from("Tooltip"), |ui| {
            let block_or_item = player_inventory.items.get(from_slot).unwrap().as_ref().unwrap().block_or_item;
            let coords = extracted_items.request(block_or_item).unwrap_or((0.0, 0.0));
            egui::Image::new(inventory_texture.1, [95.0, 95.0]).uv([[coords.0, coords.1].into(), [coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0].into()]).ui(ui);
          });
        }
        ui.style_mut().spacing.button_padding = emath::Vec2::ZERO;
        let mut swap = None;
        egui::Grid::new("Inventory Grid").show(ui, |ui| {
          for (i, item) in player_inventory.items.iter().enumerate().dropping(player_inventory.hot_bar_width) {
            let color = Color32::DARK_BLUE;
            if let ItemMove::FromSlot(from_slot) = *item_move && from_slot == i {
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
                match *item_move {
                  ItemMove::Nothing => {
                    if item.is_some() {
                      *item_move = ItemMove::FromSlot(i);
                    }
                  }
                  ItemMove::FromSlot(from_slot) => {
                    swap = Some((from_slot, i));
                  }
                }
              }
            }
            if i % player_inventory.hot_bar_width == player_inventory.hot_bar_width - 1 {
              ui.end_row();
            }
          }
        });
        ui.separator();
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
          for (i, item) in player_inventory.items.iter().enumerate().take(player_inventory.hot_bar_width) {
            let color = Color32::DARK_BLUE;
            if let ItemMove::FromSlot(from_slot) = *item_move && from_slot == i {
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
                match *item_move {
                  ItemMove::Nothing => {
                    if item.is_some() {
                      *item_move = ItemMove::FromSlot(i);
                    }
                  }
                  ItemMove::FromSlot(from_slot) => {
                    swap = Some((from_slot, i));
                  }
                }
              }
            }
          }
        });
        if let Some((from, to)) = swap {
          *item_move = ItemMove::Nothing;
          player_inventory.items.swap(from, to);
        }
      });
  }
}
