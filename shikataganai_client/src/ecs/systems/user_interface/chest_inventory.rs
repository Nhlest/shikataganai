use crate::ecs::plugins::client::Requested;
use crate::ecs::plugins::imgui::GUITextureAtlas;
use crate::ecs::plugins::rendering::inventory_pipeline::ExtractedItems;
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use crate::ecs::systems::user_interface::{item_button, ButtonStyle};
use crate::ImguiState;
use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use bincode::serialize;
use imgui::{Condition, StyleVar};
use shikataganai_common::ecs::components::blocks::{QuantifiedBlockOrItem, ReverseLocation};
use shikataganai_common::ecs::components::functors::InternalInventory;
use shikataganai_common::networking::{ClientChannel, FunctorType, PlayerCommand};
use std::iter::Rev;

pub struct InventoryOpened(pub Entity);

pub fn chest_inventory(
  mut commands: Commands,
  imgui: NonSendMut<ImguiState>,
  window: Res<Windows>,
  mut inventory_opened: Option<ResMut<InventoryOpened>>,
  texture: Res<GUITextureAtlas>,
  hotbar_items: Res<PlayerInventory>,
  selected_hotbar: Res<SelectedHotBar>,
  extracted_items: Res<ExtractedItems>,
  mut inventory_query: Query<&mut InternalInventory>,
  mut requested_query: Query<&Requested>,
  mut location_query: Query<&ReverseLocation>,
  mut client: ResMut<RenetClient>,
) {
  const ITEM_WIDTH: f32 = 50.0;
  if let Some(inventory_entity) = inventory_opened {
    if let Ok(internal_inventory) = inventory_query.get_mut(inventory_entity.0) {
      let active_window = window.get_primary().unwrap();
      let ui = imgui.get_current_frame();
      // TODO: clamp and shit
      let width = active_window.width() - 500.0;
      let height = active_window.height() - 300.0;
      imgui::Window::new("Chest inventory")
        .title_bar(false)
        .resizable(false)
        .scrollable(false)
        .scroll_bar(false)
        .position(
          [
            active_window.width() / 2.0 - width / 2.0,
            (active_window.height() - height) / 2.0,
          ],
          Condition::Always,
        )
        .size([width, height], Condition::Always)
        .build(ui, || {
          let _a = ui.push_style_var(StyleVar::ItemSpacing([2.5, 2.5]));
          let number_of_buttons_per_row = (width / (ITEM_WIDTH + 5.0)).floor();
          let left_margin = (width - number_of_buttons_per_row * (ITEM_WIDTH + 5.0) - 2.5) / 2.0;
          ui.set_cursor_pos([left_margin, 2.5]);
          let mut items_in_current_row = 0;
          for item in internal_inventory.inventory.iter() {
            item_button(
              ui,
              [95.0, 95.0],
              item.as_ref(),
              texture.as_ref(),
              extracted_items.as_ref(),
              ButtonStyle::Normal,
            );
          }
        })
        .unwrap();
    } else {
      if requested_query.get(inventory_entity.0).is_ok() {
        return;
      }
      if let Ok(location) = location_query.get(inventory_entity.0) {
        commands.entity(inventory_entity.0).insert(Requested);
        client.send_message(
          ClientChannel::ClientCommand.id(),
          serialize(&PlayerCommand::RequestFunctor {
            location: location.0,
            functor: FunctorType::InternalInventory,
          })
          .unwrap(),
        );
      }
    }
  }
}
