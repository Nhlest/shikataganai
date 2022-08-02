use crate::ecs::plugins::camera::MainMenuOpened;
use crate::ecs::plugins::imgui::{BigFont, GUITextureAtlas};
use crate::ecs::plugins::settings::{AmbientOcclusion, FullScreen, MouseSensitivity, Resolution, VSync};
use crate::ecs::resources::player::PlayerInventory;
use crate::ImguiState;
use bevy::app::AppExit;
use bevy::prelude::*;
use imgui::{ComboBoxPreviewMode, Condition, StyleVar};

pub fn hot_bar(
  imgui: NonSendMut<ImguiState>,
  window: Res<Windows>,
  texture: Res<GUITextureAtlas>,
  hotbar_items: Res<PlayerInventory>,
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
      for (_i, _item) in hotbar_items.items.iter().enumerate() {
        imgui::Image::new(texture.0, [95.0, 95.0])
          .uv0([1.0 / 8.0, 0.0])
          .uv1([2.0 / 8.0, 1.0 / 8.0])
          .border_col([0.0, 0.0, 0.0, 1.0])
          .build(&ui);
        ui.same_line();
      }
    })
    .unwrap();
}

pub fn main_menu(
  imgui: NonSendMut<ImguiState>,
  mut window: ResMut<Windows>,
  mut main_menu_opened: ResMut<MainMenuOpened>,
  mut settings_menu_opened: Local<bool>,
  mut app_exit: EventWriter<AppExit>,
  big_font: NonSend<BigFont>,
  mut mouse_sensetivity: ResMut<MouseSensitivity>,
  mut resolution: ResMut<Resolution>,
  mut vsync: ResMut<VSync>,
  mut fullscreen: ResMut<FullScreen>,
  mut ambient_occlusion: ResMut<AmbientOcclusion>,
) {
  if !main_menu_opened.0 {
    return;
  }
  let active_window = window.get_primary_mut().unwrap();
  let ui = imgui.get_current_frame();

  imgui::Window::new("Main Menu")
    // .title_bar(false)
    .resizable(false)
    .scrollable(false)
    .scroll_bar(false)
    .position(
      [
        active_window.width() as f32 / 2.0 - 150.0,
        active_window.height() as f32 / 2.0 - 250.0,
      ],
      Condition::FirstUseEver,
    )
    .size([300.0, 500.0], Condition::Always)
    .build(ui, || {
      let _f = ui.push_font(big_font.0);
      let [x1, _] = ui.window_content_region_min();
      let [x2, _] = ui.window_content_region_max();
      let w = ui.calc_text_size("Continue");
      ui.set_cursor_pos([((x2 - x1) - w[0]) / 2.0, ui.cursor_pos()[1]]);
      if ui.button("Continue") {
        main_menu_opened.0 = false;
        active_window.set_cursor_lock_mode(true);
        active_window.set_cursor_visibility(false);
      }
      let w = ui.calc_text_size("Settings");
      ui.set_cursor_pos([((x2 - x1) - w[0]) / 2.0, ui.cursor_pos()[1]]);
      if ui.button("Settings") {
        *settings_menu_opened = true;
      }
      let w = ui.calc_text_size("Exit");
      ui.set_cursor_pos([((x2 - x1) - w[0]) / 2.0, ui.cursor_pos()[1]]);
      if ui.button("Exit") {
        app_exit.send(AppExit)
      }
    })
    .unwrap();
  if !*settings_menu_opened {
    return;
  }
  imgui::Window::new("Settings Menu")
    .title_bar(false)
    .resizable(false)
    .scrollable(false)
    .scroll_bar(false)
    .position(
      [
        active_window.width() / 2.0 - 200.0,
        active_window.height() / 2.0 - 300.0,
      ],
      Condition::Always,
    )
    .size([400.0, 600.0], Condition::Always)
    .build(ui, || {
      let _f = ui.push_font(big_font.0);
      let [x1, _] = ui.window_content_region_min();
      let [x2, _] = ui.window_content_region_max();

      imgui::Slider::new("Sensitivity", 0.0, 2.0).build(ui, &mut mouse_sensetivity.as_mut().0);
      let selected = format!("{}x{}", resolution.width as i32, resolution.height as i32);
      let tok = imgui::ComboBox::new("Resolution")
        .preview_mode(ComboBoxPreviewMode::Full)
        .preview_value(selected.as_str())
        .begin(ui);
      match tok {
        None => {}
        Some(_) => {
          for (x, y) in [
            (320, 180),
            (640, 360),
            (853, 480),
            (1280, 720),
            (1920, 1080),
            (2560, 1444),
            (3840, 2160),
          ] {
            let current = format!("{}x{}", x, y);
            if imgui::Selectable::new(current.as_str())
              .selected(current == selected)
              .build(ui)
            {
              resolution.width = x as f32;
              resolution.height = y as f32;
              window
                .get_primary_mut()
                .unwrap()
                .set_resolution(resolution.width, resolution.height);
            }
          }
        }
      }
      if ui.checkbox("Vi Sink", &mut vsync.as_mut().0) {
        window
          .get_primary_mut()
          .unwrap()
          .set_present_mode(vsync.as_present_mode());
      }
      if ui.checkbox("Fullscreen", &mut fullscreen.as_mut().0) {
        window.get_primary_mut().unwrap().set_mode(fullscreen.as_mode());
      }
      ui.checkbox("Ambient Occlusion", &mut ambient_occlusion.as_mut().0);
      let w = ui.calc_text_size("Close");
      ui.set_cursor_pos([((x2 - x1) - w[0]) / 2.0, ui.cursor_pos()[1]]);
      if ui.button("Close") {
        *settings_menu_opened = false;
      }
    })
    .unwrap();
}
