use bevy::prelude::Entity;

use crate::ecs::plugins::client::Requested;
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_renet::renet::RenetClient;
use bincode::serialize;
use shikataganai_common::ecs::components::blocks::ReverseLocation;
use shikataganai_common::ecs::components::functors::InternalInventory;
use shikataganai_common::networking::{ClientChannel, FunctorType, PlayerCommand};

#[derive(Resource)]
pub struct InventoryOpened(pub Entity);
//
// #[derive(Default)]
// pub enum InventoryItemMovementStatus {
//   #[default]
//   Nothing,
//   HoldingItemFrom(usize),
// }
//
pub fn chest_inventory(
  mut commands: Commands,
  egui: ResMut<EguiContext>,
  // window: Res<Windows>,
  inventory_opened: Option<ResMut<InventoryOpened>>,
  // texture: Res<GUITextureAtlas>,
  mut extracted_items: ResMut<ExtractedItems>,
  inventory_query: Query<&mut InternalInventory>,
  requested_query: Query<&Requested>,
  location_query: Query<&ReverseLocation>,
  mut client: ResMut<RenetClient>,
) {
  if let Some(inventory_entity) = inventory_opened.map(|e| e.0) {
    match inventory_query.get(inventory_entity) {
      Ok(internal_inventory) => {
        // let ui = imgui.get_current_frame();
        // imgui::Window::new("Chest inventory")
        //   .position([20.0, 20.0], Condition::Appearing)
        //   .size([800.0, 600.0], Condition::Appearing)
        //   .build(ui, || {
        //     ui.set_cursor_pos([2.0, 2.0 + 25.0]);
        //     render_item_grid(
        //       ui,
        //       (5, 2),
        //       |x, y| (internal_inventory.inventory[y * 5 + x].as_ref(), x + y),
        //       texture.as_ref(),
        //       extracted_items.as_mut(),
        //     );
        //   });
      }
      Err(_) => {
        if !requested_query.get(inventory_entity).is_ok() {
          let location = location_query.get(inventory_entity).unwrap();
          client.send_message(
            ClientChannel::ClientCommand.id(),
            serialize(&PlayerCommand::RequestFunctor {
              location: location.0,
              functor: FunctorType::InternalInventory,
            })
            .unwrap(),
          );
          commands.entity(inventory_entity).insert(Requested);
        }
      }
    }
  }
}
