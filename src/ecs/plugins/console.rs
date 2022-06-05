use bevy::app::Plugin;
use tracing::Level;
use bevy::ecs::event::Events;
use bevy::input::InputSystem;
use crate::ecs::systems::input::{DebugMenuOpened};
use crate::ecs::plugins::imgui::{BigFont, ImguiFrameSystem};
use crate::ecs::plugins::settings;
use crate::ImguiState;
use bevy::prelude::*;
use imgui::{ComboBoxPreviewMode, Condition, StyleVar};
use crate::App;
use crate::ecs::plugins::settings::Resolution;

pub  struct ConsolePlugin;

pub enum ConsoleCommandEvents{
    ToggleNoClip {enabled: bool},
    SpawnLight,
}
impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_to_stage(CoreStage::PreUpdate, (debug_console).after(ImguiFrameSystem))
            .add_event::<ConsoleCommandEvents>();
    }
}

pub fn debug_console(
    imgui: NonSendMut<ImguiState>,
    mut window: ResMut<Windows>,
    debug_console_opened: ResMut<DebugMenuOpened>,
    big_font: NonSend<BigFont>,
    mut items: Local<Vec<String>>,
    mut console_event_writer: EventWriter<ConsoleCommandEvents>,
) {
    if !debug_console_opened.0 { return; }
    let active_window = window.get_primary_mut().unwrap();
    let ui = imgui.get_current_frame();
    
    fn add_log(items: &mut Vec<String>, log: &str, level: Level) {
        let mut text = log.to_string();
        items.push(text);
    }
    imgui::Window::new("Debug Console")
        .position(
            [
                0 as f32 ,
                0 as f32 ,
            ],
            Condition::Once,
        )
        .size([active_window.width(), 300.0], Condition::Once)
        .movable(false).collapsible(false).resizable(false).title_bar(false).focus_on_appearing(true)
        .build(ui, || {
            let _f = ui.push_font(big_font.0);
           
            let cwindow = imgui::ChildWindow::new(0)
                .border(false).draw_background(true).size([active_window.width(),250.0])
                .begin(ui);
            match cwindow {
                None => {}
                Some(_) => {
                    for item in items.iter() {
                        let mut item_text = item.to_string();
                        let mut item_text_size = ui.calc_text_size(item_text.as_str());
                        ui.text(item_text);
                    }
                }
            }
            drop(cwindow);
            let mut buff0 = String::with_capacity(32);
            let input_width = ui.push_item_width(active_window.width() - 10 as f32);
            if ui.is_window_focused() && !ui.is_any_item_focused() && !ui.is_mouse_clicked(imgui::MouseButton::Left){
                ui.set_keyboard_focus_here();
            }
            if ui.input_text("##", &mut buff0)
                .enter_returns_true(true).auto_select_all(true).allow_tab_input(true).hint("input commands")
                .build(){
                let command = buff0.trim();
                if command == "clear"{
                    items.clear();
                }
                if  command == "noclip"{
                    add_log(&mut items, "noclip", Level::INFO);
                    console_event_writer.send(ConsoleCommandEvents::ToggleNoClip {enabled: true});
                }
            }
            input_width.pop(ui);
        })
        .unwrap();
}
