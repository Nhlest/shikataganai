use crate::ecs::components::block::BlockId;
use crate::ecs::plugins::camera::Selection;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::{RelightEvent, RelightType};
use crate::ecs::resources::chunk_map::{BlockAccessor, BlockAccessorStatic};
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use crate::util::array::DDD;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::prelude::{Collider, InteractionGroups, RapierContext};
use tracing::field::debug;

pub fn action_input(
  mouse: Res<Input<MouseButton>>,
  selection: Res<Option<Selection>>,
  // hotbar_items: Res<PlayerInventory>,
  // hotbar_selection: Res<SelectedHotBar>,
  mut block_accessor: BlockAccessorStatic,
  mut relight_events: EventWriter<RelightEvent>,
  rapier_context: Res<RapierContext>,
) {
  // let hotbar_selection = &hotbar_items.items[hotbar_selection.0 as usize];
  match selection.into_inner() {
    None => {}
    Some(Selection { cube, face }) => {
      let source: DDD = *cube;
      let target_negative = *face;
      // let (dx, dy, dz) = (
      //   source.0 - target_negative.0,
      //   source.1 - target_negative.1,
      //   source.2 - target_negative.2,
      // );
      // let _target_positive = add_ddd(source, (dx, dy, dz));
      // let up = (source.0, source.1 + 1, source.2);
      // let down = (source.0, source.1 - 1, source.2);
      if mouse.just_pressed(MouseButton::Left) {
        if let Some([source_block]) = block_accessor.get_many_mut([source]) {
          source_block.block = BlockId::Air;
          //TODO: remesh neighbouring chunks
          relight_events.send(RelightEvent::Relight(RelightType::BlockRemoved, source));
        }
      }
      if mouse.just_pressed(MouseButton::Right) {
        if let Some([target_negative_block]) = block_accessor.get_many_mut([target_negative]) {
          let shape = Collider::cuboid(0.45, 0.45, 0.45);
          let shape_pos = Vec3::new(
            target_negative.0 as f32 + 0.5,
            target_negative.1 as f32 + 0.5,
            target_negative.2 as f32 + 0.5,
          );
          let shape_rot = Quat::IDENTITY;
          if rapier_context
            .intersection_with_shape(
              shape_pos,
              shape_rot,
              &shape,
              QueryFilter {
                flags: Default::default(),
                groups: Some(InteractionGroups::new(0b10, 0b01)),
                exclude_collider: None,
                exclude_rigid_body: None,
                predicate: None,
              },
            )
            .is_none()
          {
            target_negative_block.block = BlockId::Cobble;
            relight_events.send(RelightEvent::Relight(RelightType::BlockAdded, target_negative));
          }
        }
      }
    }
  }
}

pub fn hot_bar_scroll_input(
  mut selected_hotbar: ResMut<SelectedHotBar>,
  mut scroll_wheel: EventReader<MouseWheel>,
  hotbar_items: Res<PlayerInventory>,
  keys: Res<Input<KeyCode>>,
) {
  let hotbar_length = hotbar_items.items.len() as i32;
  for MouseWheel { y, .. } in scroll_wheel.iter() {
    selected_hotbar.0 = selected_hotbar.0 - *y as i32;
    selected_hotbar.0 = (hotbar_length + (selected_hotbar.0 % hotbar_length)) % hotbar_length;
  }
  if keys.just_pressed(KeyCode::Key1) {
    selected_hotbar.0 = 0;
  }
  if keys.just_pressed(KeyCode::Key2) {
    selected_hotbar.0 = 1;
  }
  if keys.just_pressed(KeyCode::Key3) {
    selected_hotbar.0 = 2;
  }
}
pub struct MainMenuOpened(pub bool);
pub struct ConsoleMenuOpened(pub bool);

pub fn handle_menu_system(
  mut windows: ResMut<Windows>,
  key: Res<Input<KeyCode>>,
  mut main_menu_opened: ResMut<MainMenuOpened>,
  mut debug_menu_opened: ResMut<ConsoleMenuOpened>,
) {
  let window = windows.get_primary_mut().unwrap();

  if key.just_pressed(KeyCode::Escape) {
    if main_menu_opened.0 {
      window.set_cursor_lock_mode(true);
      window.set_cursor_visibility(false);
    } else {
      window.set_cursor_lock_mode(false);
      window.set_cursor_visibility(true);
    }
    main_menu_opened.0 = !main_menu_opened.0;
  }
  if key.just_pressed(KeyCode::Grave) {
    window.set_cursor_lock_mode(debug_menu_opened.0);
    window.set_cursor_visibility(!debug_menu_opened.0);
    debug_menu_opened.0 = !debug_menu_opened.0;
  }
}
