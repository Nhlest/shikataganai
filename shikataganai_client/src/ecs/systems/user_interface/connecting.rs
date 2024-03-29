use bevy::prelude::*;
use bevy_egui::EguiContext;

pub fn connecting_window(mut egui: ResMut<EguiContext>) {
  let ui = egui.ctx_mut();
  egui::Window::new("Connecting...").show(ui, |ui| {
    ui.label("Connecting...");
  });
}
