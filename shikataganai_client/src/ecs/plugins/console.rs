use crate::ecs::plugins::game::{in_game, LocalTick};
use crate::App;
use bevy::app::Plugin;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use iyes_loopless::prelude::ConditionSet;
use tracing::Level;

pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
  fn build(&self, app: &mut App) {
    let on_game_simulation_last = ConditionSet::new().run_if(in_game).with_system(commit_log_lines).into();
    let on_game_simulation_continuous = ConditionSet::new()
      .run_if(in_game)
      .with_system(open_close_console)
      .with_system(debug_console)
      .into();
    app
      .add_event::<ConsoleText>()
      .init_resource::<ConsoleMenuOpened>()
      .init_resource::<ConsoleTextVec>()
      .add_system_set_to_stage(CoreStage::Last, on_game_simulation_last)
      .add_system_set(on_game_simulation_continuous);
  }
}

#[derive(Clone)]
pub struct ConsoleText {
  pub text: String,
  pub level: Level,
  pub age: u64,
}

#[derive(Clone, Default, Resource, Deref, DerefMut)]
pub struct ConsoleTextVec(pub Vec<ConsoleText>);

#[derive(Default, Resource)]
pub struct ConsoleMenuOpened(pub bool);

pub fn commit_log_lines(mut events: EventReader<ConsoleText>, mut lines: ResMut<ConsoleTextVec>) {
  lines.extend(events.iter().cloned());
}

pub fn open_close_console(
  mut windows: ResMut<Windows>,
  key: Res<Input<KeyCode>>,
  mut debug_menu_opened: ResMut<ConsoleMenuOpened>,
) {
  let window = windows.get_primary_mut().unwrap();
  if key.just_pressed(KeyCode::Grave) {
    window.set_cursor_grab_mode(if debug_menu_opened.0 {
      CursorGrabMode::Locked
    } else {
      CursorGrabMode::None
    });
    window.set_cursor_visibility(!debug_menu_opened.0);
    debug_menu_opened.0 = !debug_menu_opened.0;
  }
}

pub fn debug_console(
  mut window: ResMut<Windows>,
  debug_console_opened: ResMut<ConsoleMenuOpened>,
  mut items: ResMut<ConsoleTextVec>,
  tick: Res<LocalTick>,
) {
  // let active_window = window.get_primary_mut().unwrap();
  // let ui = imgui.get_current_frame();
  // if !debug_console_opened.0 {
  //   imgui::Window::new("Debug Console Messages")
  //     .position([0.0, 0.0], Condition::Always)
  //     .size([active_window.width(), active_window.height()], Condition::Always)
  //     .movable(false)
  //     .collapsible(false)
  //     .resizable(false)
  //     .title_bar(false)
  //     .focus_on_appearing(false)
  //     .focused(false)
  //     .draw_background(false)
  //     .build(ui, || {
  //       ui.set_scroll_y(9999.0);
  //       for item in items.iter().filter(|item| item.age + 1000 > tick.0) {
  //         const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
  //         const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
  //         const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
  //
  //         let token = match item.level {
  //           Level::ERROR => ui.push_style_color(StyleColor::Text, RED),
  //           Level::INFO => ui.push_style_color(StyleColor::Text, GREEN),
  //           Level::WARN => ui.push_style_color(StyleColor::Text, YELLOW),
  //           Level::TRACE => ui.push_style_color(StyleColor::Text, YELLOW),
  //           Level::DEBUG => ui.push_style_color(StyleColor::Text, YELLOW),
  //         };
  //         ui.text(&item.text);
  //         token.pop();
  //       }
  //     })
  //     .unwrap();
  //   return;
  // }
  //
  // fn add_log(items: &mut Vec<ConsoleText>, log: &str, m_level: Level, tick: &LocalTick) {
  //   let ct = ConsoleText {
  //     text: log.to_string(),
  //     level: m_level,
  //     age: **tick,
  //   };
  //   items.push(ct);
  // }
  // imgui::Window::new("Debug Console")
  //   .position([0.0, 0.0], Condition::Always)
  //   .size([active_window.width(), 300.0], Condition::Always)
  //   .movable(false)
  //   .collapsible(false)
  //   .resizable(false)
  //   .title_bar(false)
  //   .focus_on_appearing(true)
  //   .build(ui, || {
  //     let _f = ui.push_font(big_font.0);
  //     {
  //       let cwindow = imgui::ChildWindow::new(0)
  //         .border(false)
  //         .draw_background(true)
  //         .size([active_window.width(), 250.0])
  //         .begin(ui);
  //       if cwindow.is_some() {
  //         ui.set_scroll_y(9999.0); // TODO: lock/unlock autoscroll with manual override
  //         for item in items.iter() {
  //           const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
  //           const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
  //           const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
  //
  //           let token = match item.level {
  //             Level::ERROR => ui.push_style_color(StyleColor::Text, RED),
  //             Level::INFO => ui.push_style_color(StyleColor::Text, GREEN),
  //             Level::WARN => ui.push_style_color(StyleColor::Text, YELLOW),
  //             Level::TRACE => ui.push_style_color(StyleColor::Text, YELLOW),
  //             Level::DEBUG => ui.push_style_color(StyleColor::Text, YELLOW),
  //           };
  //           ui.text(&item.text);
  //           token.pop();
  //         }
  //       }
  //     }
  //     let mut buff0 = String::with_capacity(32);
  //     let input_width = ui.push_item_width(active_window.width() - 10.0);
  //     if ui.is_window_focused() && !ui.is_any_item_focused() && !ui.is_mouse_clicked(imgui::MouseButton::Left) {
  //       ui.set_keyboard_focus_here();
  //     }
  //     if ui
  //       .input_text("##", &mut buff0)
  //       .enter_returns_true(true)
  //       .auto_select_all(true)
  //       .allow_tab_input(true)
  //       .hint("input commands")
  //       .build()
  //     {
  //       fn get_bool_command(command: &str) -> bool {
  //         command == "1"
  //       }
  //       #[inline]
  //       fn get_float_command(command: &str) -> f32 {
  //         command.parse::<f32>().unwrap_or(0.0)
  //       }
  //       let command = buff0.split_whitespace().collect::<Vec<&str>>();
  //       match command.first() {
  //         Some(..) => match command[0] {
  //           "clear" => {
  //             items.clear();
  //           }
  //           _ => match command.get(1) {
  //             Some(..) => match command[0] {
  //               "noclip" => {
  //                 // let cd = get_bool_command(command[1]);
  //                 add_log(
  //                   &mut items,
  //                   std::format!("noclip {}", command[1]).as_str(),
  //                   Level::INFO,
  //                   &tick,
  //                 );
  //               }
  //               "player_speed" => {
  //                 let cd = get_float_command(command[1]);
  //                 if cd == 0.0 {
  //                   return;
  //                 }
  //                 add_log(
  //                   &mut items,
  //                   std::format!("Changed player speed to: {}", command[1]).as_str(),
  //                   Level::INFO,
  //                   &tick,
  //                 );
  //               }
  //               _ => {
  //                 add_log(
  //                   &mut items,
  //                   std::format!("Error: Couldn't find the command '{}'", command[0]).as_str(),
  //                   Level::ERROR,
  //                   &tick,
  //                 );
  //               }
  //             },
  //             None => return,
  //           },
  //         },
  //         None => return,
  //       }
  //     }
  //     input_width.pop(ui);
  //   })
  //   .unwrap();
}
