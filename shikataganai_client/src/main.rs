#![feature(const_fn_floating_point_arithmetic)]
#![feature(let_chains)]
#![feature(negative_impls)]
#![feature(array_methods)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![allow(irrefutable_let_patterns)]

use bevy::app::AppLabel;
#[allow(unused_imports)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::{default, App, ImagePlugin, PluginGroup, WindowDescriptor};
use bevy::window::WindowPlugin;
use bevy::DefaultPlugins;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_egui::EguiPlugin;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_framepace::FramepacePlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};

use crate::ecs::plugins::camera::CameraPlugin;
use crate::ecs::plugins::client::ShikataganaiClientPlugin;
use crate::ecs::plugins::console::ConsolePlugin;
use crate::ecs::plugins::game::GamePlugin;
use crate::ecs::plugins::preamble::Preamble;
use crate::ecs::plugins::rendering::mesh_pipeline::loader::GltfMeshStorage;
use crate::ecs::plugins::rendering::ShikataganaiRendererPlugins;
use crate::ecs::plugins::settings::{FullScreen, Resolution, SettingsPlugin, VSync};

mod ecs;

fn main() {
  let mut app = App::new();
  app.add_plugin(SettingsPlugin).add_plugin(Preamble);
  let resolution = app.world.resource::<Resolution>();
  let vsync = app.world.resource::<VSync>();
  let fullscreen = app.world.resource::<FullScreen>();
  app
    .add_plugins(
      DefaultPlugins
        .set(WindowPlugin {
          window: WindowDescriptor {
            width: resolution.width,
            height: resolution.height,
            resizable: true,
            title: "仕方がない、ね？".to_string(),
            present_mode: vsync.as_present_mode(),
            mode: fullscreen.as_mode(),
            ..default()
          },
          add_primary_window: true,
          exit_on_all_closed: true,
          close_when_requested: true,
        })
        .set(ImagePlugin::default_nearest())
        .build()
        .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
    )
    .add_plugin(AtmospherePlugin)
    .add_plugin(GamePlugin)
    .add_plugin(CameraPlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
    .add_plugin(EguiPlugin)
    .add_plugins(ShikataganaiRendererPlugins)
    .add_plugin(ConsolePlugin)
    .add_plugin(FramepacePlugin)
    .add_plugin(ShikataganaiClientPlugin)
    // .add_plugin(RapierDebugRenderPlugin::default())
    // .add_plugin(LogDiagnosticsPlugin::default())
    // .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .run();
}
