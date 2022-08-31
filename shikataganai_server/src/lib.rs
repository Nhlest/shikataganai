use bevy::prelude::*;

use crate::ecs::plugins::server::{ShikataganaiServerAddress, ShikataganaiServerPlugin};

pub mod ecs;

pub fn spawn_server(address: ShikataganaiServerAddress) {
  App::new()
    .add_plugins(MinimalPlugins)
    .insert_resource(address)
    .add_plugin(ShikataganaiServerPlugin)
    .run();
}
