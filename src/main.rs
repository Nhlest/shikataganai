#![feature(negative_impls)]

use bevy::prelude::{App, NonSend};
use bevy::DefaultPlugins;
use parry3d::na::{Isometry3, Vector3};

use crate::ecs::plugins::camera::CameraPlugin;
use crate::ecs::plugins::game::GamePlugin;
use crate::ecs::plugins::imgui::{ImguiPlugin, ImguiState};
use crate::ecs::plugins::preamble::Preamble;
use crate::ecs::plugins::voxel::VoxelRendererPlugin;

mod ecs;
mod util;

fn main() {
  fn gui(imgui: NonSend<ImguiState>) {
    let mut ui = imgui.get_current_frame();
    imgui::Window::new("Pepega").build(&mut ui, || {}).unwrap();
  }

  App::new()
    .add_plugin(Preamble)
    .add_plugins(DefaultPlugins)
    .add_plugin(CameraPlugin)
    .add_plugin(GamePlugin)
    // .add_plugin(LogDiagnosticsPlugin::default())
    // .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(ImguiPlugin)
    .add_plugin(VoxelRendererPlugin)
    .add_system(gui)
    .run();
}
