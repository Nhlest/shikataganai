#![feature(const_fn_floating_point_arithmetic)]
#![feature(negative_impls)]
#![feature(vec_into_raw_parts)]

use crate::ecs::plugins::animation::AnimationPlugin;
#[allow(unused_imports)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::App;
use bevy::DefaultPlugins;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};

use crate::ecs::plugins::camera::CameraPlugin;
use crate::ecs::plugins::game::GamePlugin;
use crate::ecs::plugins::imgui::{ImguiPlugin, ImguiState};
use crate::ecs::plugins::preamble::Preamble;
use crate::ecs::plugins::settings::SettingsPlugin;
use crate::ecs::plugins::voxel::VoxelRendererPlugin;

mod ecs;
mod util;

fn main() {
  App::new()
    .add_plugin(SettingsPlugin)
    .add_plugin(Preamble)
    .add_plugins_with(DefaultPlugins, |group| {
      group.add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin)
    })
    .add_plugin(CameraPlugin)
    .add_plugin(GamePlugin)
    .add_plugin(AnimationPlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(ImguiPlugin)
    .add_plugin(VoxelRendererPlugin)
    // .add_plugin(bevy_framepace::FramepacePlugin::default())
    // .add_plugin(RapierDebugRenderPlugin::default())
    // .add_plugin(LogDiagnosticsPlugin::default())
    // .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .run();
}
