use crate::ecs::plugins::camera::MainMenuOpened;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::ecs::resources::player::{HotBarItems, SelectedHotBar};
use crate::ecs::systems::chunkgen::collect_async_chunks;
use crate::ecs::systems::input::{action_input, hot_bar_scroll_input};
use crate::ecs::systems::ui::{cursor, hot_bar, main_menu};
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_system(action_input)
      .add_system(hot_bar_scroll_input)
      .add_system(hot_bar)
      .add_system(cursor)
      .add_system(main_menu.exclusive_system())
      .add_system(collect_async_chunks)
      .insert_resource(MainMenuOpened(true))
      .init_resource::<SelectedHotBar>()
      .init_resource::<HotBarItems>()
      .init_resource::<ChunkMap>();
  }
}
