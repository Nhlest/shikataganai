use crate::ecs::plugins::game::ShikataganaiGameState;
use crate::ecs::plugins::imgui::BigFont;
use crate::App;
use crate::ImguiState;
use bevy::app::Plugin;
use bevy::prelude::*;
use imgui::{Condition, StyleColor};
use iyes_loopless::prelude::{ConditionSet, CurrentState};
use tracing::Level;

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
  fn build(&self, app: &mut App) {
    let on_game_simulation_continuous = ConditionSet::new()
      .run_if(|state: Option<Res<CurrentState<ShikataganaiGameState>>>| {
        state
          .map(|state| state.0 == ShikataganaiGameState::Simulation || state.0 == ShikataganaiGameState::Paused)
          .unwrap_or(false)
      })
      .with_system(open_close_console)
      .with_system(debug_console)
      .into();
    app
      .init_resource::<ConsoleMenuOpened>()
      .add_system_set(on_game_simulation_continuous);
  }
}

pub struct ConsoleText {
  text: String,
  level: Level,
}

#[derive(Default)]
pub struct ConsoleMenuOpened(pub bool);

pub fn open_close_console(
  mut windows: ResMut<Windows>,
  key: Res<Input<KeyCode>>,
  mut debug_menu_opened: ResMut<ConsoleMenuOpened>,
) {
  let window = windows.get_primary_mut().unwrap();
  if key.just_pressed(KeyCode::Grave) {
    window.set_cursor_lock_mode(debug_menu_opened.0);
    window.set_cursor_visibility(!debug_menu_opened.0);
    debug_menu_opened.0 = !debug_menu_opened.0;
  }
}

pub fn debug_console(
  imgui: NonSendMut<ImguiState>,
  mut window: ResMut<Windows>,
  debug_console_opened: ResMut<ConsoleMenuOpened>,
  big_font: NonSend<BigFont>,
  mut items: Local<Vec<ConsoleText>>,
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
    .position([0.0, 0.0], Condition::Always)
    .size([active_window.width(), 300.0], Condition::Always)
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
              &Level::ERROR => ui.push_style_color(StyleColor::Text, RED),
              &Level::INFO => ui.push_style_color(StyleColor::Text, GREEN),
              &Level::WARN => ui.push_style_color(StyleColor::Text, YELLOW),
              &Level::TRACE => ui.push_style_color(StyleColor::Text, YELLOW),
              &Level::DEBUG => ui.push_style_color(StyleColor::Text, YELLOW),
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
                  // let cd = get_bool_command(command[1]);
                  add_log(&mut items, std::format!("noclip {}", command[1]).as_str(), Level::INFO);
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
