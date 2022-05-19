use crate::ecs::plugins::imgui::GUITextureAtlas;
use crate::ecs::resources::player::{HotBarItems, SelectedHotBar};
use crate::ecs::resources::ui::UiSprite;
use crate::ImguiState;
use bevy::prelude::*;
use imgui::{Condition, StyleVar};

pub fn hot_bar(
  imgui: NonSend<ImguiState>,
  window: Res<Windows>,
  texture: Res<GUITextureAtlas>,
  selected_hotbar: Res<SelectedHotBar>,
  hotbar_items: Res<HotBarItems>,
) {
  let active_window = window.get_primary().unwrap();
  let ui = imgui.get_current_frame();
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
      let _a = ui.push_style_var(StyleVar::ItemSpacing([2.5, 2.5]));
      for (i, item) in hotbar_items.items.iter().enumerate() {
        let c = ui.cursor_pos();
        let sprite = if i as i32 == selected_hotbar.0 {
          UiSprite::BlueSquare
        } else {
          UiSprite::RedSquare
        };
        imgui::Image::new(texture.0, [95.0, 95.0])
          .uv0(sprite.into_uv().0)
          .uv1(sprite.into_uv().1)
          .border_col([0.0, 0.0, 0.0, 1.0])
          .build(&ui);
        ui.set_cursor_pos(c);
        let sprite = item.sprite();
        imgui::Image::new(texture.0, [95.0, 95.0])
          .uv0(sprite.into_uv().0)
          .uv1(sprite.into_uv().1)
          .border_col([0.0, 0.0, 0.0, 1.0])
          .build(&ui);
        ui.same_line();
      }
    })
    .unwrap();
}

pub fn cursor(imgui: NonSend<ImguiState>, window: Res<Windows>) {
  let active_window = window.get_primary().unwrap();
  let ui = imgui.get_current_frame();
  let _a = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));
  let _b = ui.push_style_var(StyleVar::WindowMinSize([0.0, 0.0]));
  let _c = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
  imgui::Window::new("Cursor")
    .title_bar(false)
    .resizable(false)
    .scrollable(false)
    .scroll_bar(false)
    .position(
      [active_window.width() / 2.0 - 2.0, active_window.height() / 2.0 - 2.0],
      Condition::Always,
    )
    .size([4.0, 4.0], Condition::Always)
    .build(ui, || {})
    .unwrap();
}
