use crate::ecs::plugins::imgui::{BigFont, ImguiFrameSystem};
use crate::ecs::systems::input::ConsoleMenuOpened;
use crate::App;
use crate::ImguiState;
use bevy::app::Plugin;
use bevy::prelude::*;
use imgui::{Condition, StyleColor};
use tracing::Level;

pub struct ConsolePlugin;
pub enum ConsoleCommandEvents {
  ToggleNoClip(bool),
  PlayerSpeed(f32),
}
impl Plugin for ConsolePlugin {
  fn build(&self, app: &mut App) {
    app.add_system(debug_console).add_event::<ConsoleCommandEvents>();
  }
}
pub struct ConsoleText {
  text: String,
  level: Level,
}
pub fn debug_console(
  imgui: NonSendMut<ImguiState>,
  mut window: ResMut<Windows>,
  debug_console_opened: ResMut<ConsoleMenuOpened>,
  big_font: NonSend<BigFont>,
  mut items: Local<Vec<ConsoleText>>,
  mut console_event_writer: EventWriter<ConsoleCommandEvents>,
) {
  if !debug_console_opened.0 {
    return;
  }

  let active_window = window.get_primary_mut().unwrap();
  let ui = imgui.get_current_frame();
  fn add_log(items: &mut Vec<ConsoleText>, log: &str, m_level: Level) {
    let ct = ConsoleText {
      text: log.to_string(),
      level: m_level,
    };
    items.push(ct);
  }
  imgui::Window::new("Debug Console")
    .position([0.0, 0.0], Condition::Once)
    .size([active_window.width(), 300.0], Condition::Once)
    .movable(false)
    .collapsible(false)
    .resizable(false)
    .title_bar(false)
    .focus_on_appearing(true)
    .build(ui, || {
      let _f = ui.push_font(big_font.0);
      {
        let cwindow = imgui::ChildWindow::new(0)
          .border(false)
          .draw_background(true)
          .size([active_window.width(), 250.0])
          .begin(ui);
        if cwindow.is_some() {
          for item in items.iter() {
            const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
            const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
            const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

            let token = match &item.level {
              &Level::ERROR =>  ui.push_style_color(StyleColor::Text, RED),
              &Level::INFO =>   ui.push_style_color(StyleColor::Text, GREEN),
              &Level::WARN =>   ui.push_style_color(StyleColor::Text, YELLOW),
              &Level::TRACE =>  ui.push_style_color(StyleColor::Text, YELLOW),
              &Level::DEBUG =>  ui.push_style_color(StyleColor::Text, YELLOW),
            };
            ui.text(&item.text);
            token.pop();
          }
        }
      }
      let mut buff0 = String::with_capacity(32);
      let input_width = ui.push_item_width(active_window.width() - 10 as f32);
      if ui.is_window_focused() && !ui.is_any_item_focused() && !ui.is_mouse_clicked(imgui::MouseButton::Left) {
        ui.set_keyboard_focus_here();
      }
      if ui
        .input_text("##", &mut buff0)
        .enter_returns_true(true)
        .auto_select_all(true)
        .allow_tab_input(true)
        .hint("input commands")
        .build()
      {
        fn get_bool_command(command: &str) -> bool {
          return command == "1";
        }
        #[inline]
        fn get_float_command(command: &str) -> f32 {
          command.parse::<f32>().unwrap_or(0.0)
        }
        let command = buff0.split_whitespace().collect::<Vec<&str>>();
        match command.get(0) {
          Some(..) => match command[0] {
            "clear" => {
              items.clear();
            }
            _ => match command.get(1) {
              Some(..) => match command[0] {
                "noclip" => {
                  let cd = get_bool_command(command[1]);
                  add_log(&mut items, std::format!("noclip {}", command[1]).as_str(), Level::INFO);
                  console_event_writer.send(ConsoleCommandEvents::ToggleNoClip(cd));
                }
                "player_speed" => {
                  let cd = get_float_command(command[1]);
                  if cd == 0.0 {
                    return;
                  }
                  add_log(
                    &mut items,
                    std::format!("Changed player speed to: {}", command[1]).as_str(),
                    Level::INFO,
                  );
                  console_event_writer.send(ConsoleCommandEvents::PlayerSpeed(cd))
                }
                _ => {
                  add_log(
                    &mut items,
                    std::format!("Error: Couldn't find the command '{}'", command[0]).as_str(),
                    Level::ERROR,
                  );
                }
              },
              None => return,
            },
          },
          None => return,
        }
      }
      input_width.pop(ui);
    })
    .unwrap();
}
