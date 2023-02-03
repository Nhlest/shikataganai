use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::plugins::rendering::inventory_pipeline::InventoryTextureOutputHandle;
use bevy::prelude::{Res, ResMut, Windows};
use bevy_egui::EguiContext;
use egui::{Color32, LayerId, Response, Sense, TextStyle, Ui, Widget};
use shikataganai_common::ecs::components::blocks::QuantifiedBlockOrItem;
use std::ops::Range;

pub mod chest_inventory;
pub mod connecting;
pub mod game_menu;
pub mod hot_bar;
pub mod main_menu;
pub mod player_inventory;

pub fn item_button(
  ui: &mut Ui,
  content: Option<&QuantifiedBlockOrItem>,
  extracted_items: &mut ExtractedItems,
  inventory_texture: &InventoryTextureOutputHandle,
) -> Response {
  let mut response = match content {
    None => egui::ImageButton::new(inventory_texture.1, [95.0, 95.0])
      .uv([[1.0, 1.1].into(), [1.0, 1.0].into()])
      .sense(Sense::click_and_drag())
      .ui(ui),
    Some(QuantifiedBlockOrItem { block_or_item, quant }) => {
      ui.allocate_ui([95.0, 95.0].into(), |ui| {
        let text = egui::WidgetText::RichText(egui::RichText::new(format!("{}", quant))).into_galley(
          ui,
          None,
          50.0,
          TextStyle::Button,
        );
        let coords = extracted_items.request(*block_or_item).unwrap_or((0.0, 0.0));
        let pos = ui.next_widget_position();
        let sense = egui::ImageButton::new(inventory_texture.1, [95.0, 95.0])
          .uv([
            [coords.0, coords.1].into(),
            [coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0].into(),
          ])
          .sense(Sense::click_and_drag())
          .ui(ui);
        text.paint_with_fallback_color(ui.painter(), pos, Color32::WHITE);
        sense
      })
      .inner
    }
  };
  if response.drag_started() {
    response.clicked[0] = true;
  }
  response
}

fn item_button_grid<'a, F>(
  id: impl std::hash::Hash,
  ui: &mut Ui,
  content_fetch: F,
  cell_range: Range<usize>,
  grid_width: usize,
  extracted_items: &mut ExtractedItems,
  inventory_texture: &InventoryTextureOutputHandle,
) -> Option<usize>
where
  F: Fn(usize) -> Option<&'a QuantifiedBlockOrItem>,
{
  let mut clicked = None;
  egui::Grid::new(id).show(ui, |ui| {
    for celli in cell_range {
      let item = content_fetch(celli);
      if item_button(ui, item, extracted_items, inventory_texture).clicked() {
        clicked = Some(celli)
      }
      if celli % grid_width == grid_width - 1 {
        ui.end_row();
      }
    }
  });
  clicked
}

pub fn cursor_marker(mut egui: ResMut<EguiContext>, window: Res<Windows>) {
  let h = window.primary().height() / 2.0;
  let w = window.primary().width() / 2.0;
  egui
    .ctx_mut()
    .layer_painter(LayerId::background())
    .hline(w - 4.0..=w + 4.0, h, (2.0, Color32::DARK_GRAY));
  egui
    .ctx_mut()
    .layer_painter(LayerId::background())
    .vline(w, h - 4.0..=h + 4.0, (2.0, Color32::DARK_GRAY));
}
