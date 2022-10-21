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
use std::ops::Deref;

pub struct InventoryOpened(pub Entity);

#[derive(Default)]
pub enum InventoryItemMovementStatus {
  #[default]
  Nothing,
  HoldingItemFrom(usize)
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
  extracted_items: Res<ExtractedItems>,
  mut inventory_query: Query<&mut InternalInventory>,
  mut requested_query: Query<&Requested>,
  mut location_query: Query<&ReverseLocation>,
  mut client: ResMut<RenetClient>,
) {
  const ITEM_WIDTH: f32 = 50.0;
  if let Some(inventory_entity) = inventory_opened {
    if let Ok(mut internal_inventory) = inventory_query.get_mut(inventory_entity.0) {
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
        .bring_to_front_on_focus(false)
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

          enum Action {
            MoveAmongst(usize, usize),
          }

          let mut action = None;
          for (position, item) in internal_inventory.inventory.iter().enumerate() {
            let same_row = if items_in_current_row >= 8 {
              items_in_current_row = 0;
              false
            } else {
              items_in_current_row += 1;
              true
            };
            let style = match inventory_item_movement_status.deref() {
              InventoryItemMovementStatus::HoldingItemFrom(item) if *item == position => ButtonStyle::Highlight,
              _ => ButtonStyle::Normal,
            };
            let item = if let InventoryItemMovementStatus::HoldingItemFrom(item) = inventory_item_movement_status.deref() && *item == position {
              &None
            } else {
              item
            };
            if item_button(
              ui,
              [95.0, 95.0],
              item.as_ref(),
              texture.as_ref(),
              extracted_items.as_ref(),
              style,
              same_row,
              position
            ) {
              match *inventory_item_movement_status {
                InventoryItemMovementStatus::Nothing => {
                  if item.is_some() {
                    *inventory_item_movement_status = InventoryItemMovementStatus::HoldingItemFrom(position);
                  }
                }
                InventoryItemMovementStatus::HoldingItemFrom(position_from) => {
                  action = Some(Action::MoveAmongst(position_from, position));
                }
              }
            }
          }
          match action {
            None => {}
            Some(Action::MoveAmongst(from, to)) => {
              if from != to {
                let from_item =
                  internal_inventory.inventory
                    .get(from)
                    .unwrap()
                    .as_ref()
                    .map(|x| x.block_or_item);
                let to_item =
                  internal_inventory.inventory
                    .get(to)
                    .unwrap()
                    .as_ref()
                    .map(|x| x.block_or_item);
                if to_item == from_item {
                  let from_quant = internal_inventory.inventory
                    .get(from)
                    .unwrap()
                    .as_ref()
                    .map(|x| x.quant);
                  from_quant.map(|quant| {
                    internal_inventory.inventory
                      .get_mut(to)
                      .unwrap()
                      .as_mut()
                      .map(|x| x.quant += quant);
                  });
                  *internal_inventory.inventory.get_mut(from).unwrap() = None;
                } else {
                  internal_inventory.inventory.swap(from, to);
                }
              }
              *inventory_item_movement_status = InventoryItemMovementStatus::Nothing;
            }
          }
        })
        .unwrap();
      if let InventoryItemMovementStatus::HoldingItemFrom(from) = inventory_item_movement_status.deref() {
        let mouse = active_window.cursor_position().unwrap();
        imgui::Window::new("Drag-drop tooltip")
          .title_bar(false)
          .resizable(false)
          .scrollable(false)
          .scroll_bar(false)
          .draw_background(false)
          .position(
            [
              mouse.x + 2.0,
              active_window.height() - mouse.y + 2.0
            ],
            Condition::Always,
          )
          .size([100.0, 100.0], Condition::Always)
          .build(ui, || {
            let item = internal_inventory.inventory.get(*from).unwrap();
            item_button(
              ui,
              [95.0, 95.0],
              item.as_ref(),
              texture.as_ref(),
              extracted_items.as_ref(),
              ButtonStyle::Normal,
              true,
              999
            )
          }).unwrap();
      }
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
