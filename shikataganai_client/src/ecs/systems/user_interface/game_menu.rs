use crate::ecs::plugins::game::ShikataganaiGameState;
use crate::ecs::plugins::settings::{AmbientOcclusion, FullScreen, MouseSensitivity, Resolution, VSync};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_rapier3d::plugin::RapierConfiguration;
use egui::Widget;
use iyes_loopless::prelude::NextState;
use std::ops::RangeInclusive;

pub fn game_menu(
  mut commands: Commands,
  mut egui: ResMut<EguiContext>,
  mut window: ResMut<Windows>,
  mut settings_menu_opened: Local<bool>,
  mut app_exit: EventWriter<AppExit>,
  mut mouse_sensitivity: ResMut<MouseSensitivity>,
  mut resolution: ResMut<Resolution>,
  mut vsync: ResMut<VSync>,
  mut fullscreen: ResMut<FullScreen>,
  mut ambient_occlusion: ResMut<AmbientOcclusion>,
  mut physics_system: ResMut<RapierConfiguration>,
) {
  egui::Window::new("Main Menu").show(egui.ctx_mut(), |ui| {
    if ui.button("Continue").clicked() {
      physics_system.physics_pipeline_active = true;
      commands.insert_resource(NextState(ShikataganaiGameState::Simulation));
    }
    if ui.button("Settings").clicked() {
      *settings_menu_opened = true;
    }
    if ui.button("Exit").clicked() {
      app_exit.send(AppExit)
    }
  });
  if !*settings_menu_opened {
    return;
  }
  egui::Window::new("Settings Menu").show(egui.ctx_mut(), |ui| {
    egui::Slider::new(&mut mouse_sensitivity.as_mut().0, RangeInclusive::new(0.0, 2.0)).ui(ui);
    let selected = format!("{}x{}", resolution.width as i32, resolution.height as i32);
    let mut selection = (resolution.width as i32, resolution.height as i32);
    if egui::ComboBox::from_label("Resolution")
      .selected_text(selected)
      .show_ui(ui, |ui| {
        for s in [
          (320, 180),
          (640, 360),
          (853, 480),
          (1280, 720),
          (1920, 1080),
          (2560, 1444),
          (3840, 2160),
        ] {
          ui.selectable_value(&mut selection, s, format!("{:?}", s));
        }
      })
      .response
      .clicked()
    {
      //TODO: doesnt really work
      resolution.width = selection.0 as f32;
      resolution.height = selection.1 as f32;
      window
        .get_primary_mut()
        .unwrap()
        .set_resolution(resolution.width, resolution.height);
    }
    if ui.checkbox(&mut vsync.as_mut().0, "Vi Sink").changed() {
      window
        .get_primary_mut()
        .unwrap()
        .set_present_mode(vsync.as_present_mode());
    }
    if ui.checkbox(&mut fullscreen.as_mut().0, "Fullscreen").changed() {
      window.get_primary_mut().unwrap().set_mode(fullscreen.as_mode());
    }
    ui.checkbox(&mut ambient_occlusion.as_mut().0, "Ambient Occlusion");
    if ui.button("Close").clicked() {
      *settings_menu_opened = false;
    }
  });
}
