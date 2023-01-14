use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use bevy::prelude::*;
use bevy_egui::EguiContext;
use egui::{Align, Layout, Widget};
use shikataganai_common::ecs::components::blocks::QuantifiedBlockOrItem;
use crate::ecs::plugins::rendering::inventory_pipeline::GUITextureAtlas;

pub fn hot_bar(
  mut egui: ResMut<EguiContext>,
  window: Res<Windows>,
  // texture: Res<GUITextureAtlas>,
  hotbar_items: Res<PlayerInventory>,
  selected_hotbar: Res<SelectedHotBar>,
  mut extracted_items: ResMut<ExtractedItems>,
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
          // let cursor = ui.cursor_screen_pos();
          // let c = ui.cursor_pos();
          // if i as i32 == selected_hotbar.0 {
          //   ui.get_background_draw_list()
          //     .add_rect(cursor, [cursor[0] + 95.0, cursor[1] + 95.0], [1.0, 0.0, 0.0, 0.8])
          //     .filled(true)
          //     .build();
          // } else {
          //   ui.get_background_draw_list()
          //     .add_rect(cursor, [cursor[0] + 95.0, cursor[1] + 95.0], [0.0, 0.0, 1.0, 0.8])
          //     .filled(true)
          //     .build();
          // }
          // ui.set_cursor_pos(c);
          // match item {
          //   None => {
          //     egui::Image::new(texture.0, [95.0, 95.0]).uv([[1.0, 1.0].into(), [1.0, 1.0].into()]).ui(ui);
          //   }
          //   Some(QuantifiedBlockOrItem { block_or_item, quant }) => {
          //     let coords = extracted_items.request(*block_or_item).unwrap_or((0.0, 0.0));
          //     egui::Image::new(texture.0, [95.0, 95.0]).uv([[coords.0, coords.1].into(), [coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0].into()]).ui(ui);
          //     // ui.text(format!("{}", quant));
          //   }
          // }
        }
      });
    });
  //     ui.get_background_draw_list()
  //       .add_rect([x1, y1], [x1 + 4.0, y1 + 4.0], [0.1, 0.1, 0.1, 1.0])
  //       .build();
  //     let _a = ui.push_style_var(StyleVar::ItemSpacing([2.5, 2.5]));
  //   })
  //   .unwrap();
}
