use crate::ecs::components::block::{Block, BlockId};
use crate::ecs::components::chunk::Chunk;
use crate::ecs::plugins::camera::Selection;
use crate::ecs::plugins::voxel::Remesh;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::ecs::resources::light::Relight;
use crate::ecs::resources::player::{HotBarItem, HotBarItems, SelectedHotBar};
use crate::util::array::DDD;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;

pub fn action_input(
  mouse: Res<Input<MouseButton>>,
  selection: Res<Option<Selection>>,
  mut chunks: Query<&mut Chunk>,
  mut chunk_map: ResMut<ChunkMap>,
  mut relight: ResMut<Relight>,
  mut remesh: ResMut<Remesh>,
  hotbar_items: Res<HotBarItems>,
  hotbar_selection: Res<SelectedHotBar>,
  mut commands: Commands,
  dispatcher: Res<AsyncComputeTaskPool>,
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
      let target_positive = (source.0 + dx, source.1 + dy, source.2 + dz);
      let up = (source.0, source.1 + 1, source.2);
      let down = (source.0, source.1 - 1, source.2);
      match hotbar_selection {
        HotBarItem::PushPull => {
          if mouse.just_pressed(MouseButton::Left) {
            if let Some([source, target_positive, down]) =
              chunk_map.get_many_mut(&mut commands, &dispatcher, &mut chunks, [source, target_positive, down])
            {
              if target_positive.block == BlockId::Air {
                let block = std::mem::replace(source, Block { block: BlockId::Air });
                let _ = std::mem::replace(target_positive, block);
                if down.block == BlockId::Hoist {
                  let _ = std::mem::replace(down, Block { block: BlockId::Air });
                }
                remesh.remesh();
                relight.relight();
              }
            }
          }
          if mouse.just_pressed(MouseButton::Right) {
            if let Some([source, target_negative, down]) =
              chunk_map.get_many_mut(&mut commands, &dispatcher, &mut chunks, [source, target_negative, down])
            {
              if target_negative.block == BlockId::Air {
                let block = std::mem::replace(source, Block { block: BlockId::Air });
                let _ = std::mem::replace(target_negative, block);
                if down.block == BlockId::Hoist {
                  let _ = std::mem::replace(down, Block { block: BlockId::Air });
                }
                remesh.remesh();
                relight.relight();
              }
            }
          }
        }
        HotBarItem::HoistUnhoist => {
          if mouse.just_pressed(MouseButton::Left) {
            if let Some([source, up, down]) =
              chunk_map.get_many_mut(&mut commands, &dispatcher, &mut chunks, [source, up, down])
            {
              if down.block != BlockId::Hoist && up.block == BlockId::Air {
                let block = std::mem::replace(source, Block { block: BlockId::Hoist });
                let _ = std::mem::replace(up, block);
                remesh.remesh();
                relight.relight();
              }
            }
          }
          if mouse.just_pressed(MouseButton::Right) {
            if let Some([source, down]) =
              chunk_map.get_many_mut(&mut commands, &dispatcher, &mut chunks, [source, down])
            {
              if down.block == BlockId::Hoist {
                let block = std::mem::replace(source, Block { block: BlockId::Air });
                let _ = std::mem::replace(down, block);
                remesh.remesh();
                relight.relight();
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
