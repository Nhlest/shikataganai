use std::f32::consts::LOG10_2;
use bevy::app::Plugin;
use tracing::Level;
use bevy::ecs::event::Events;
use bevy::ecs::schedule::ShouldRun::No;
use bevy::input::InputSystem;
use crate::ecs::systems::input::{DebugMenuOpened};
use crate::ecs::plugins::imgui::{BigFont, ImguiFrameSystem};
use crate::ImguiState;
use bevy::prelude::*;
use imgui::{ComboBoxPreviewMode, Condition, StyleVar};
use bevy_rapier3d::na::{Vector4, Vector3};
use tracing::field::debug;
use crate::App;

pub  struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_to_stage(CoreStage::PreUpdate, (debug_console).after(ImguiFrameSystem))
            .add_event::<NoClipEvent>();
    }
}
pub enum ConsoleCommands{
    ToggleNoClip,
    SpawnLight,
}
pub struct NoClipEvent {
    pub enabled: bool,
}
pub fn debug_console(
    imgui: NonSendMut<ImguiState>,
    mut window: ResMut<Windows>,
    debug_console_opened: ResMut<DebugMenuOpened>,
    big_font: NonSend<BigFont>,
    mut items: Local<Vec<String>>,
) {
    let mut events = Events::<NoClipEvent>::default();
    if !debug_console_opened.0 { return; }
    let active_window = window.get_primary_mut().unwrap();
    let ui = imgui.get_current_frame();
    let mut reader = events.get_reader();
    
    // let mut demo_bool = true;
    // ui.show_demo_window(&mut demo_bool);
    
    fn add_log(items: &mut Vec<String>, log: &str, level: Level) {
        let mut text = log.to_string();
        items.push(text);
    }
    imgui::Window::new("Debug Console")
        .position(
            [
                active_window.width() as f32 / 2.0 ,
                active_window.height() as f32 / 2.0,
            ],
            Condition::Once,
        )
        .size([500.0, 300.0], Condition::Once)
        .build(ui, || {
            let _f = ui.push_font(big_font.0);

            ui.same_line();
            if ui.button("Clear") { //clear the items
                items.clear();
            }
            //child window for displaying items(logs)
            let mut window_width = 400.0;
            let cwindow = imgui::ChildWindow::new(0).border(true).draw_background(true).size([window_width,200.0]).begin(ui);
            match cwindow {
                None => {}
                Some(_) => {
                    for item in items.iter() {
                        let mut item_text = item.to_string();
                        // item_text.push_str("\n");
                        let mut item_text_size   = ui.calc_text_size(item_text.as_str());
                        if item_text_size[0] > window_width {
                            window_width = item_text_size[0];
                        }
                        ui.text(item_text.as_str());
                    }
                }
            }
            drop(cwindow);
            let mut buff0 = String::with_capacity(64);
            if ui.input_text("input command", &mut buff0).enter_returns_true(true).build(){
                let command = buff0.trim();
                if command == "rust sucks ass" {
                    add_log(&mut items, "indeed it does xD ", Level::INFO);
                }
                if command == "rust is good and makes you feel like a genius"{
                    add_log(&mut items, "glueless", Level::INFO);
                }
                if  command == "noclip"{
                    add_log(&mut items, "noclip", Level::INFO);
                    events.send(NoClipEvent{enabled: true});
                }
            }
        })
        .unwrap();
}
