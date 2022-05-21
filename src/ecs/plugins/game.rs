use crate::ecs::components::light::LightSource;
use crate::ecs::components::Location;
use crate::ecs::plugins::camera::MainMenuOpened;
use crate::ecs::resources::chunk_map::{ChunkMap, ChunkMapSize};
use crate::ecs::resources::light::LightMap;
use crate::ecs::resources::player::{HotBarItems, SelectedHotBar};
use crate::ecs::systems::input::{action_input, hot_bar_scroll_input};
use crate::ecs::systems::light::light_system;
use crate::ecs::systems::ui::{cursor, hot_bar, main_menu};
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_system(light_system)
      .add_system(action_input)
      .add_system(hot_bar_scroll_input)
      .add_system(hot_bar)
      .add_system(cursor)
      .add_system(main_menu.exclusive_system())
      .insert_resource(ChunkMapSize { x: 5, y: 5 })
      .insert_resource(MainMenuOpened(true))
      .init_resource::<SelectedHotBar>()
      .init_resource::<HotBarItems>()
      .init_resource::<LightMap>()
      .init_resource::<ChunkMap>();
    app.world.spawn().insert(LightSource).insert(Location::new(16, 15, 16));
  }
}
