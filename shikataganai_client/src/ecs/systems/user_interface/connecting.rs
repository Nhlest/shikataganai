use crate::ImguiState;
use bevy::prelude::*;
use imgui::Condition;

pub fn connecting_window(imgui: NonSend<ImguiState>, mut window: ResMut<Windows>) {
  let active_window = window.get_primary_mut().unwrap();
  let ui = imgui.get_current_frame();

  imgui::Window::new("Connecting...")
    .resizable(false)
    .scrollable(false)
    .scroll_bar(false)
    .position(
      [
        active_window.width() as f32 / 2.0 - 150.0,
        active_window.height() as f32 / 2.0 - 25.0,
      ],
      Condition::FirstUseEver,
    )
    .size([300.0, 50.0], Condition::Always)
    .build(ui, || {
      ui.label_text("Connecting...", "Connecting...");
    })
    .unwrap();
}
