use crate::ecs::components::block::{Block, BlockId};
use crate::ecs::components::chunk::Chunk;
use crate::ecs::components::Location;
use crate::ecs::plugins::camera::Selection;
use crate::ecs::plugins::voxel::Remesh;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::ecs::resources::light::Relight;
use crate::ecs::resources::player::{HotBarItem, HotBarItems, SelectedHotBar};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

pub fn action_input(
  mouse: Res<Input<MouseButton>>,
  selection: Res<Option<Selection>>,
  mut chunks: Query<&mut Chunk>,
  chunk_map: Res<ChunkMap>,
  mut relight: ResMut<Relight>,
  mut remesh: ResMut<Remesh>,
  hotbar_items: Res<HotBarItems>,
  hotbar_selection: Res<SelectedHotBar>,
) {
  let hotbar_selection = &hotbar_items.items[hotbar_selection.0 as usize];
  match selection.into_inner() {
    None => {}
    Some(Selection { cube, face }) => {
      let source = Location::from(cube);
      let target_negative = Location::from(face);
      let (dx, dy, dz) = (
        source.x - target_negative.x,
        source.y - target_negative.y,
        source.z - target_negative.z,
      );
      let target_positive = Location::new(source.x + dx, source.y + dy, source.z + dz);
      let up = Location::new(source.x, source.y + 1, source.z);
      let down = Location::new(source.x, source.y - 1, source.z);
      match hotbar_selection {
        HotBarItem::PushPull => {
          if mouse.just_released(MouseButton::Left) {
            if let (Some((e, c)), Some((e2, c2)), Some((e3, c3))) = (
              chunk_map.get_path_to_block_location(source),
              chunk_map.get_path_to_block_location(target_positive),
              chunk_map.get_path_to_block_location(down),
            ) {
              if chunks.get(e2).unwrap().grid[c2.into()].block == BlockId::Air {
                let block = std::mem::replace(
                  &mut chunks.get_mut(e).unwrap().grid[c.into()],
                  Block { block: BlockId::Air },
                );
                let _ = std::mem::replace(&mut chunks.get_mut(e2).unwrap().grid[c2.into()], block);
                if chunks.get(e3).unwrap().grid[c3.into()].block == BlockId::Hoist {
                  let _ = std::mem::replace(
                    &mut chunks.get_mut(e3).unwrap().grid[c3.into()],
                    Block { block: BlockId::Air },
                  );
                }
                remesh.remesh();
                relight.relight();
              }
            }
          }
          if mouse.just_released(MouseButton::Right) {
            if let (Some((e, c)), Some((e2, c2)), Some((e3, c3))) = (
              chunk_map.get_path_to_block_location(source),
              chunk_map.get_path_to_block_location(target_negative),
              chunk_map.get_path_to_block_location(down),
            ) {
              if chunks.get(e2).unwrap().grid[c2.into()].block == BlockId::Air {
                let block = std::mem::replace(
                  &mut chunks.get_mut(e).unwrap().grid[c.into()],
                  Block { block: BlockId::Air },
                );
                let _ = std::mem::replace(&mut chunks.get_mut(e2).unwrap().grid[c2.into()], block);
                if chunks.get(e3).unwrap().grid[c3.into()].block == BlockId::Hoist {
                  let _ = std::mem::replace(
                    &mut chunks.get_mut(e3).unwrap().grid[c3.into()],
                    Block { block: BlockId::Air },
                  );
                }
                remesh.remesh();
                relight.relight();
              }
            }
          }
        }
        HotBarItem::HoistUnhoist => {
          if mouse.just_released(MouseButton::Left) {
            if let (Some((e, c)), Some((e2, c2)), Some((e3, c3))) = (
              chunk_map.get_path_to_block_location(source),
              chunk_map.get_path_to_block_location(up),
              chunk_map.get_path_to_block_location(down),
            ) {
              if chunks.get(e3).unwrap().grid[c3.into()].block != BlockId::Hoist
                && chunks.get(e2).unwrap().grid[c2.into()].block == BlockId::Air
              {
                let block = std::mem::replace(
                  &mut chunks.get_mut(e).unwrap().grid[c.into()],
                  Block { block: BlockId::Hoist },
                );
                let _ = std::mem::replace(&mut chunks.get_mut(e2).unwrap().grid[c2.into()], block);
                remesh.remesh();
                relight.relight();
              }
            }
          }
          if mouse.just_released(MouseButton::Right) {
            if let (Some((e, c)), Some((e3, c3))) = (
              chunk_map.get_path_to_block_location(source),
              chunk_map.get_path_to_block_location(down),
            ) {
              if chunks.get(e3).unwrap().grid[c3.into()].block == BlockId::Hoist {
                let block = std::mem::replace(
                  &mut chunks.get_mut(e).unwrap().grid[c.into()],
                  Block { block: BlockId::Air },
                );
                let _ = std::mem::replace(&mut chunks.get_mut(e3).unwrap().grid[c3.into()], block);
                remesh.remesh();
                relight.relight();
              }
            }
          }
        }
        HotBarItem::Delete => {}
        HotBarItem::Empty => {}
      }
    } // None => {}
      // Some(Selection { cube, face }) => {
      //   if mouse.just_pressed(MouseButton::Left) {
      //     if let Some((e, c)) = chunk_map.get_path_to_block_location(Location::new(cube[0], cube[1], cube[2])) {
      //       chunks.get_mut(e).unwrap().grid[c.into()].block = BlockId::Air;
      //       relight.0 = true;
      //       remesh.0 = true;
      //     }
      //   }
      //   if mouse.just_pressed(MouseButton::Right) {
      //     if let Some((e, c)) = chunk_map.get_path_to_block_location(Location::new(face[0], face[1], face[2])) {
      //       chunks.get_mut(e).unwrap().grid[c.into()].block = BlockId::Dirt;
      //       relight.0 = true;
      //       remesh.0 = true;
      //     }
      //   }
      // }
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
  if keys.just_released(KeyCode::Key1) {
    selected_hotbar.0 = 0;
  }
  if keys.just_released(KeyCode::Key2) {
    selected_hotbar.0 = 1;
  }
  if keys.just_released(KeyCode::Key3) {
    selected_hotbar.0 = 2;
  }
}
