use bevy::prelude::*;
use imgui::Condition;
use crate::ecs::plugins::imgui::{GUITextureAtlas, ImguiState};
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::resources::player::PlayerInventory;
use crate::ecs::systems::user_interface::render_item_grid;

pub struct PlayerInventoryOpened;

pub fn player_inventory(
  mut commands: Commands,
  inventory_opened: Option<ResMut<PlayerInventoryOpened>>,
  player_inventory: ResMut<PlayerInventory>,
  imgui: NonSendMut<ImguiState>,
  texture: Res<GUITextureAtlas>,
  mut extracted_items: ResMut<ExtractedItems>,
) {
  if let Some(_) = inventory_opened {
    let ui = imgui.get_current_frame();
    imgui::Window::new("Player inventory")
      .position([20.0, 20.0], Condition::Appearing)
      .size([800.0, 600.0], Condition::Appearing)
      .build(ui, || {
        let size_y = (player_inventory.items.len() - player_inventory.hot_bar_width) / player_inventory.hot_bar_width;

        ui.set_cursor_pos([2.0, 2.0 + 25.0]);

        render_item_grid(
          ui,
          (player_inventory.hot_bar_width, size_y),
          |x, y| (player_inventory.items[y * player_inventory.hot_bar_width + x + player_inventory.hot_bar_width].as_ref(), x + y),
          texture.as_ref(),
          extracted_items.as_mut(),
        );

        ui.set_cursor_pos([2.0, ui.window_size()[1] - 100.0]);

        render_item_grid(
          ui,
          (player_inventory.hot_bar_width, 1),
          |x, y| (player_inventory.items[y * player_inventory.hot_bar_width + x].as_ref(), 999 + x + y),
          texture.as_ref(),
          extracted_items.as_mut(),
        );
      });
  }
}