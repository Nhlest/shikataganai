use crate::ecs::components::block::{Block, BlockId};
use crate::ecs::plugins::camera::Selection;
use crate::ecs::plugins::voxel::{RelightEvent, RelightType};
use crate::ecs::resources::chunk_map::{BlockAccessor, BlockAccessorStatic};
use crate::ecs::resources::player::{HotBarItem, HotBarItems, SelectedHotBar};
use crate::util::array::{add_ddd, DDD};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, InteractionGroups, RapierContext};

pub fn action_input(
  mouse: Res<Input<MouseButton>>,
  selection: Res<Option<Selection>>,
  hotbar_items: Res<HotBarItems>,
  hotbar_selection: Res<SelectedHotBar>,
  mut block_accessor: BlockAccessorStatic,
  mut relight_events: EventWriter<RelightEvent>,
  rapier_context: Res<RapierContext>,
) {
  let hotbar_selection = &hotbar_items.items[hotbar_selection.0 as usize];
  match selection.into_inner() {
    None => {}
    Some(Selection { cube, face }) => {
      let source: DDD = *cube;
      let target_negative = *face;
      let (dx, dy, dz) = (
        source.0 - target_negative.0,
        source.1 - target_negative.1,
        source.2 - target_negative.2,
      );
      let _target_positive = add_ddd(source, (dx, dy, dz));
      let up = (source.0, source.1 + 1, source.2);
      let down = (source.0, source.1 - 1, source.2);
      match hotbar_selection {
        HotBarItem::PushPull => {
          if mouse.just_pressed(MouseButton::Left) {
            if let Some([source_block]) = block_accessor.get_many_mut([source]) {
              source_block.block = BlockId::Air;
              //TODO: remesh neighbouring chunks
              relight_events.send(RelightEvent::Relight(RelightType::BlockRemoved, source));
            }
            // if let Some([target_positive_block, down_block]) = block_accessor.get_many_mut([target_positive, down]) {
            //   if target_positive_block.block == BlockId::Air {
            //     if down_block.block == BlockId::Hoist {
            //       let _ = std::mem::replace(down_block, Block::new(BlockId::Air));
            //     }
            //     // chunk_map.animate(source, target_positive, &mut commands_dispatcher.commands, &mut chunks, BlockId::Air);
            //   }
            // }
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
              let groups = InteractionGroups::new(0b10, 0b01);
              let filter = None;
              if rapier_context
                .intersection_with_shape(shape_pos, shape_rot, &shape, groups, filter)
                .is_none()
              {
                target_negative_block.block = BlockId::Cobble;
                relight_events.send(RelightEvent::Relight(RelightType::BlockAdded, target_negative));
              }
            }
          }
        }
        HotBarItem::HoistUnhoist => {
          if mouse.just_pressed(MouseButton::Left) {
            relight_events.send(RelightEvent::Relight(RelightType::LightSourceAdded, target_negative));
            if let Some([_source_block, up_block, down_block]) = block_accessor.get_many_mut([source, up, down]) {
              if down_block.block != BlockId::Hoist && up_block.block == BlockId::Air {
                // chunk_map.animate(source, up, &mut commands_dispatcher.commands, &mut chunks, BlockId::Hoist);
              }
            }
          }
          if mouse.just_pressed(MouseButton::Right) {
            if let Some([source, down]) = block_accessor.get_many_mut([source, down]) {
              if down.block == BlockId::Hoist {
                let block = std::mem::replace(source, Block::new(BlockId::Air));
                let _ = std::mem::replace(down, block);
              }
            }
          }
        }
        HotBarItem::Delete => {}
        HotBarItem::Empty => {}
      }
    }
  }
}

pub fn hot_bar_scroll_input(
  mut selected_hotbar: ResMut<SelectedHotBar>,
  mut scroll_wheel: EventReader<MouseWheel>,
  hotbar_items: Res<HotBarItems>,
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
