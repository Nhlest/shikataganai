use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy_egui::EguiContext;
use egui::{Align, Color32, Layout, TextStyle, TextureId, Widget};
use shikataganai_common::ecs::components::blocks::{BlockOrItem, QuantifiedBlockOrItem};
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use crate::ecs::plugins::rendering::inventory_pipeline::{GUITextureAtlas, InventoryTextureOutputHandle};

pub fn hot_bar(
  mut egui: ResMut<EguiContext>,
  window: Res<Windows>,
  // texture: Res<GUITextureAtlas>,
  hotbar_items: Res<PlayerInventory>,
  selected_hotbar: Res<SelectedHotBar>,
  mut extracted_items: ResMut<ExtractedItems>,
  inventory_texture: Res<InventoryTextureOutputHandle>
) {
  let active_window = window.get_primary().unwrap();
  let ui = egui.ctx_mut();
  let x1 = active_window.width() / 2.0 - 2.0;
  let y1 = active_window.height() / 2.0 - 2.0;
  egui::Window::new("HotBar")
    .title_bar(false)
    .resizable(false)
    .fixed_pos(
      [
        active_window.width() / 2.0 - 1080.0 / 2.0,
        active_window.height() - 100.0,
      ],
    )
    .default_size([1080.0, 100.0])
    .show(ui, |ui| {
      ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        for (i, item) in hotbar_items.items.iter().enumerate().take(hotbar_items.hot_bar_width) {
          let color = if i as i32 == selected_hotbar.0 {
            Color32::DARK_RED
          } else {
            Color32::DARK_BLUE
          };
          match item {
            None => {
              egui::Image::new(inventory_texture.1, [95.0, 95.0]).uv([[1.0, 1.0].into(), [1.0, 1.0].into()]).bg_fill(color).ui(ui);
            }
            Some(QuantifiedBlockOrItem { block_or_item, quant }) => {
              ui.allocate_ui([95.0, 95.0].into(), |ui| {
                let text = egui::WidgetText::RichText(egui::RichText::new(format!("{}", quant))).into_galley(ui, None, 50.0, TextStyle::Button);
                let coords = extracted_items.request(*block_or_item).unwrap_or((0.0, 0.0));
                let pos = ui.next_widget_position();
                egui::Image::new(inventory_texture.1, [95.0, 95.0]).uv([[coords.0, coords.1].into(), [coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0].into()]).bg_fill(color).ui(ui);
                text.paint_with_fallback_color(ui.painter(), pos, Color32::WHITE);
              });
            }
          }
        }
      });
    });
}
