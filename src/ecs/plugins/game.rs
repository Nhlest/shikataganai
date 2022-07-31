use crate::ecs::plugins::camera::{MainMenuOpened, Selection};
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use crate::ecs::systems::chunkgen::collect_async_chunks;
use crate::ecs::systems::input::{action_input, hot_bar_scroll_input};
use crate::ecs::systems::light::relight_system;
use crate::ecs::systems::ui::{hot_bar, main_menu};
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_system(action_input)
      .add_system(hot_bar_scroll_input)
      .add_system(hot_bar)
      .add_system(main_menu.exclusive_system())
      .add_system(collect_async_chunks)
      .add_system_to_stage(CoreStage::PostUpdate, relight_system)
      .insert_resource(MainMenuOpened(true))
      .init_resource::<SelectedHotBar>()
      .init_resource::<PlayerInventory>()
      .init_resource::<ChunkMap>();
  }
}
