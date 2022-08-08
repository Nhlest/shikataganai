use crate::ecs::plugins::camera::Selection;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use crate::ecs::systems::chunkgen::collect_async_chunks;
use crate::ecs::systems::input::{action_input, hot_bar_scroll_input};
use crate::ecs::systems::light::relight_system;
use crate::ecs::systems::user_interface::hot_bar::hot_bar;
use crate::ecs::systems::user_interface::main_menu::main_menu;
use bevy::prelude::*;
use std::time::Duration;

pub struct GamePlugin;

use crate::ecs::systems::user_interface::game_menu::game_menu;
use bevy::render::{Extract, RenderApp, RenderStage};
use iyes_loopless::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum ShikataganaiGameState {
  MainMenu,
  PreSimulation,
  Simulation,
  Paused,
}

#[derive(StageLabel)]
pub struct FixedUpdate;

pub fn in_game(current_state: Res<CurrentState<ShikataganaiGameState>>) -> bool {
  match current_state.0 {
    ShikataganaiGameState::Simulation | ShikataganaiGameState::Paused => true,
    _ => false,
  }
}

pub fn in_game_extract(current_state: Extract<Res<CurrentState<ShikataganaiGameState>>>) -> bool {
  match current_state.0 {
    ShikataganaiGameState::Simulation | ShikataganaiGameState::Paused => true,
    _ => false,
  }
}

pub fn init_game(mut commands: Commands) {
  commands.init_resource::<SelectedHotBar>();
  commands.init_resource::<PlayerInventory>();
  commands.init_resource::<ChunkMap>();
  commands.init_resource::<Option<Selection>>();
}

pub fn transition_to_simulation(mut commands: Commands) {
  commands.insert_resource(NextState(ShikataganaiGameState::Simulation));
}

pub fn cleanup_game(mut commands: Commands) {
  commands.remove_resource::<SelectedHotBar>();
  commands.remove_resource::<PlayerInventory>();
  commands.remove_resource::<ChunkMap>();
  commands.remove_resource::<Option<Selection>>();
}

pub fn extract_loopless_state(mut commands: Commands, state: Extract<Res<CurrentState<ShikataganaiGameState>>>) {
  commands.insert_resource(state.clone());
}

impl Plugin for GamePlugin {
  fn build(&self, app: &mut App) {
    let on_main_menu = ConditionSet::new()
      .run_in_state(ShikataganaiGameState::MainMenu)
      .with_system(main_menu)
      .into();
    let on_game_enter = SystemStage::parallel().with_system(init_game);
    let on_game_exit = SystemStage::parallel().with_system(cleanup_game);
    let on_game_pre_simulation_update = ConditionSet::new()
      .run_in_state(ShikataganaiGameState::PreSimulation)
      .with_system(transition_to_simulation)
      .into();
    let on_game_simulation_continuous = ConditionSet::new()
      .run_in_state(ShikataganaiGameState::Simulation)
      .with_system(action_input)
      .with_system(hot_bar_scroll_input)
      .with_system(hot_bar)
      .with_system(collect_async_chunks)
      .into();
    let on_pause = ConditionSet::new()
      .run_in_state(ShikataganaiGameState::Paused)
      .with_system(game_menu)
      .into();
    let on_fixed_step_simulation: SystemSet = ConditionSet::new()
      .run_in_state(ShikataganaiGameState::Simulation)
      // .with_system(|| println!("kek"))
      .into();
    let on_fixed_step_simulation_stage = SystemStage::parallel().with_system_set(on_fixed_step_simulation);
    let on_post_update_simulation = ConditionSet::new()
      .run_in_state(ShikataganaiGameState::Simulation)
      .with_system(relight_system)
      .into();

    app
      .add_loopless_state(ShikataganaiGameState::MainMenu)
      .add_stage_before(
        CoreStage::Update,
        FixedUpdate,
        FixedTimestepStage::from_stage(Duration::from_millis(125), on_fixed_step_simulation_stage),
      )
      .add_system_set(on_game_simulation_continuous)
      .add_system_set(on_main_menu)
      .add_system_set(on_game_pre_simulation_update)
      .add_system_set(on_pause)
      .add_system_set_to_stage(CoreStage::PostUpdate, on_post_update_simulation)
      .set_enter_stage(ShikataganaiGameState::MainMenu, on_game_exit)
      .set_enter_stage(ShikataganaiGameState::PreSimulation, on_game_enter);

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app.add_system_to_stage(RenderStage::Extract, extract_loopless_state);
  }
}
