use crate::ecs::plugins::imgui::GUITextureAtlas;
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use crate::ImguiState;
use bevy::prelude::*;
use imgui::{Condition, StyleVar};
use shikataganai_common::ecs::components::blocks::QuantifiedBlockOrItem;

pub fn hot_bar(
  imgui: NonSendMut<ImguiState>,
  window: Res<Windows>,
  texture: Res<GUITextureAtlas>,
  hotbar_items: Res<PlayerInventory>,
  selected_hotbar: Res<SelectedHotBar>,
  mut extracted_items: ResMut<ExtractedItems>,
) {
  let active_window = window.get_primary().unwrap();
  let ui = imgui.get_current_frame();
  let x1 = active_window.width() / 2.0 - 2.0;
  let y1 = active_window.height() / 2.0 - 2.0;
  imgui::Window::new("HotBar")
    .title_bar(false)
    .resizable(false)
    .scrollable(false)
    .scroll_bar(false)
    .position(
      [
        active_window.width() / 2.0 - 1080.0 / 2.0,
        active_window.height() - 100.0,
      ],
      Condition::Always,
    )
    .size([1080.0, 100.0], Condition::Always)
    .build(ui, || {
      ui.get_background_draw_list()
        .add_rect([x1, y1], [x1 + 4.0, y1 + 4.0], [0.1, 0.1, 0.1, 1.0])
        .build();
      let _a = ui.push_style_var(StyleVar::ItemSpacing([2.5, 2.5]));
      for (i, item) in hotbar_items.items.iter().enumerate() {
        let cursor = ui.cursor_screen_pos();
        let c = ui.cursor_pos();
        if i as i32 == selected_hotbar.0 {
          ui.get_background_draw_list()
            .add_rect(cursor, [cursor[0] + 95.0, cursor[1] + 95.0], [1.0, 0.0, 0.0, 0.8])
            .filled(true)
            .build();
        } else {
          ui.get_background_draw_list()
            .add_rect(cursor, [cursor[0] + 95.0, cursor[1] + 95.0], [0.0, 0.0, 1.0, 0.8])
            .filled(true)
            .build();
        }
        ui.set_cursor_pos(c);
        match item {
          None => {
            imgui::Image::new(texture.0, [95.0, 95.0])
              .uv0([1.0, 1.0])
              .uv1([1.0, 1.0])
              .border_col([0.0, 0.0, 0.0, 1.0])
              .build(ui);
            ui.same_line();
          }
          Some(QuantifiedBlockOrItem { block_or_item, quant }) => {
            let coords = extracted_items.request(*block_or_item).unwrap_or((0.0, 0.0));
            let cursor_before = ui.cursor_pos();
            imgui::Image::new(texture.0, [95.0, 95.0])
              .uv0([coords.0, coords.1])
              .uv1([coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0])
              .border_col([0.0, 0.0, 0.0, 1.0])
              .build(ui);
            ui.same_line();
            let cursor_after = ui.cursor_pos();
            ui.set_cursor_pos([cursor_before[0] + 80.0, cursor_before[1] + 78.0]);
            ui.text(format!("{}", quant));
            ui.same_line();
            ui.set_cursor_pos(cursor_after);
          }
        }
      }
    })
    .unwrap();
}
