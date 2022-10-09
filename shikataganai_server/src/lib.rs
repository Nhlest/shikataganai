#![feature(let_chains)]
use bevy::app::ScheduleRunnerSettings;
use bevy::prelude::*;
use shikataganai_common::ecs::resources::world::GameWorld;
use std::time::Duration;

use crate::ecs::plugins::server::{ShikataganaiServerAddress, ShikataganaiServerPlugin};

pub mod ecs;

pub fn spawn_server(address: ShikataganaiServerAddress) {
  App::new()
    .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(1.0 / 60.0)))
    .add_plugins(MinimalPlugins)
    .init_resource::<GameWorld>()
    .insert_resource(address)
    .add_plugin(ShikataganaiServerPlugin)
    .run();
}
