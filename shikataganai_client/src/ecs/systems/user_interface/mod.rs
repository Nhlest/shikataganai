use imgui::drag_drop::DragDropSourceToolTip;
use crate::ecs::plugins::imgui::GUITextureAtlas;
use crate::ecs::plugins::rendering::inventory_pipeline::ExtractedItems;
use imgui::{DragDropFlags, StyleVar, Ui};
use shikataganai_common::ecs::components::blocks::QuantifiedBlockOrItem;

pub mod chest_inventory;
pub mod connecting;
pub mod game_menu;
pub mod hot_bar;
pub mod main_menu;

pub enum ButtonStyle {
  Normal,
  Highlight,
  Active,
}

pub fn item_button(
  ui: &Ui,
  size: [f32; 2],
  item: Option<&QuantifiedBlockOrItem>,
  texture: &GUITextureAtlas,
  extracted_items: &ExtractedItems,
  style: ButtonStyle,
  same_row: bool,
  id: usize
) -> bool {
  let window = ui.window_pos();
  let cursor = ui.cursor_pos();
  let text_label = [cursor[0] + 80.0, cursor[1] + 78.0];
  let next_element = if same_row {
    [cursor[0] + size[0] + 2.0, cursor[1]]
  } else {
    [2.0, cursor[1] + size[1] + 2.0]
  };
  let bg_color = match style {
    ButtonStyle::Normal => {
      [0.5, 0.0, 0.0, 0.8]
    }
    ButtonStyle::Highlight => {
      [0.0, 0.5, 0.0, 0.8]
    }
    ButtonStyle::Active => {
      [0.0, 0.0, 0.5, 0.8]
    }
  };
  ui.set_cursor_pos(cursor);
  let clicked = match item {
    None => {
      imgui::ImageButton::new(texture.0, size)
        .uv0([1.0, 1.0])
        .uv1([1.0, 1.0])
        .build(&ui)
    }
    Some(QuantifiedBlockOrItem { block_or_item, quant }) => {
      let coords = extracted_items.0.get(&block_or_item).unwrap_or(&(0.0, 0.0)).clone();
      ui.set_cursor_pos(cursor);
      let token = ui.push_style_var(StyleVar::FramePadding([0.0, 0.0]));
      let id = ui.push_id(id as i32);
      let clicked = imgui::ImageButton::new(texture.0, size)
        .uv0([coords.0, coords.1])
        .uv1([coords.0 + 1.0 / 8.0, coords.1 + 1.0 / 8.0])
        .background_col(bg_color)
        .build(&ui);
      id.end();
      token.end();

      ui.set_cursor_pos(text_label);
      ui.text(format!("{}", quant));
      clicked
    }
  };
  ui.set_cursor_pos(next_element);
  clicked
}
