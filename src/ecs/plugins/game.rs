use crate::ecs::components::light::LightSource;
use crate::ecs::components::Location;
use crate::ecs::resources::chunk_map::{ChunkMap, ChunkMapSize};
use crate::ecs::resources::light::LightMap;
use crate::ecs::systems::input::block_input;
use crate::ecs::systems::light::light_system;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_system(light_system)
      .add_system(block_input)
      .insert_resource(ChunkMapSize { x: 5, y: 5 })
      .init_resource::<LightMap>()
      .init_resource::<ChunkMap>();
    app.world.spawn().insert(LightSource).insert(Location::new(5, 141, 5));
  }
}
