use crate::ecs::plugins::client::Requested;
use crate::ecs::plugins::imgui::GUITextureAtlas;
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use crate::ecs::systems::user_interface::{item_button, render_item_grid, ButtonStyle};
use crate::ImguiState;
use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use bincode::serialize;
use imgui::{Condition, StyleVar};
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::{BlockOrItem, QuantifiedBlockOrItem, ReverseLocation};
use shikataganai_common::ecs::components::functors::InternalInventory;
use shikataganai_common::networking::{ClientChannel, FunctorType, PlayerCommand};
use std::iter::Rev;
use std::ops::Deref;

pub struct InventoryOpened(pub Entity);

#[derive(Default)]
pub enum InventoryItemMovementStatus {
  #[default]
  Nothing,
  HoldingItemFrom(usize),
}

pub fn chest_inventory(
  mut commands: Commands,
  imgui: NonSendMut<ImguiState>,
  window: Res<Windows>,
  mut inventory_opened: Option<ResMut<InventoryOpened>>,
  texture: Res<GUITextureAtlas>,
  // hotbar_items: Res<PlayerInventory>,
  // selected_hotbar: Res<SelectedHotBar>,
  mut inventory_item_movement_status: Local<InventoryItemMovementStatus>,
  mut extracted_items: ResMut<ExtractedItems>,
  mut inventory_query: Query<&mut InternalInventory>,
  mut requested_query: Query<&Requested>,
  mut location_query: Query<&ReverseLocation>,
  mut client: ResMut<RenetClient>,
) {
  let ui = imgui.get_current_frame();
  imgui::Window::new("Chest inventory")
    .position([20.0, 20.0], Condition::Appearing)
    .size([800.0, 600.0], Condition::Appearing)
    .build(ui, || {
      render_item_grid(
        ui,
        (5, 5),
        |x, y| {
          (
            Some(&QuantifiedBlockOrItem {
              block_or_item: BlockOrItem::Block(BlockId::Stair),
              quant: 5,
            }),
            x + y,
          )
        },
        texture.as_ref(),
        extracted_items.as_mut(),
      );
    });
}
