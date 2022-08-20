use crate::ecs::components::block_or_item::BlockOrItem;
use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::components::blocks::BlockRotation;
use crate::ecs::plugins::camera::{FPSCamera, Recollide, Selection};
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::{RelightEvent, RelightType};
use crate::ecs::resources::chunk_map::{BlockAccessor, BlockAccessorStatic};
use crate::ecs::resources::player::{PlayerInventory, QuantifiedBlockOrItem, RerenderInventory, SelectedHotBar};
use crate::util::array::DDD;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::prelude::{Collider, InteractionGroups, RapierContext};
use num_traits::FloatConst;

fn place_item_from_inventory(
  player_inventory: &mut PlayerInventory,
  item_idx: usize,
  coord: DDD,
  block_accessor: &mut BlockAccessorStatic,
  rapier_context: &RapierContext,
  camera: &FPSCamera,
) {
  if let Some(Some(QuantifiedBlockOrItem {
    block_or_item: BlockOrItem::Block(block),
    quant,
  })) = player_inventory.items.get_mut(item_idx)
  {
    if let Some([target_negative_block]) = block_accessor.get_many_mut([coord]) {
      let shape = Collider::cuboid(0.5, 0.5, 0.5);
      let shape_pos = Vec3::new(coord.0 as f32 + 0.5, coord.1 as f32 + 0.5, coord.2 as f32 + 0.5);
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
        let mut phi = (camera.phi - f32::FRAC_PI_4()) % (f32::PI() * 2.0);
        if phi < 0.0 {
          phi = f32::PI() * 2.0 + phi;
        }
        if phi > 0.0 && phi <= f32::FRAC_PI_2() {
          target_negative_block.meta.set_rotation(BlockRotation::WEST);
        } else if phi > f32::FRAC_PI_2() && phi <= f32::PI() {
          target_negative_block.meta.set_rotation(BlockRotation::SOUTH);
        } else if phi > f32::PI() && phi <= f32::PI() + f32::FRAC_PI_2() {
          target_negative_block.meta.set_rotation(BlockRotation::EAST);
        } else {
          target_negative_block.meta.set_rotation(BlockRotation::NORTH);
        }
        target_negative_block.block = *block;
        *quant -= 1;
        if *quant <= 0 {
          player_inventory.items[item_idx] = None;
        }
      }
    }
  }
}

fn pick_up_block(
  mut commands: Commands,
  player_inventory: &mut PlayerInventory,
  coord: DDD,
  block_accessor: &mut BlockAccessorStatic,
  rerender_inventory: &mut ResMut<RerenderInventory>,
) {
  if let Some([source_block]) = block_accessor.get_many_mut([coord]) {
    let block: BlockId = std::mem::replace(&mut source_block.block, BlockId::Air);

    // Check for the first slot in inventory containing required block or the next empty slot to insert that block into.
    match player_inventory
      .items
      .iter_mut()
      .filter(|slot| {
        slot
          .as_ref()
          .map(|item| item.block_or_item == BlockOrItem::Block(block))
          .unwrap_or(true)
      })
      .next()
      .map(|slot| {
        slot.get_or_insert(QuantifiedBlockOrItem {
          block_or_item: BlockOrItem::Block(block),
          quant: 0,
        })
      })
      .map(|slot| {
        slot.quant += 1;
        slot.quant == 1
      }) {
      None => {
        // No empty or correct slots found, return block back into place
        let _ = std::mem::replace(&mut source_block.block, block);
        return;
      }
      Some(true) => {
        // New item added - rerender required
        rerender_inventory.0 = true;
      }
      _ => {}
    }

    if source_block.entity != Entity::from_bits(0) {
      commands.entity(source_block.entity).despawn();
      source_block.entity = Entity::from_bits(0);
    }
  }
}

pub fn action_input(
  commands: Commands,
  mouse: Res<Input<MouseButton>>,
  camera: Query<&FPSCamera>,
  selection: Res<Option<Selection>>,
  mut player_inventory: ResMut<PlayerInventory>,
  hotbar_selection: Res<SelectedHotBar>,
  mut block_accessor: BlockAccessorStatic,
  mut relight_events: EventWriter<RelightEvent>,
  rapier_context: Res<RapierContext>,
  mut rerender_inventory: ResMut<RerenderInventory>,
  mut recollide: ResMut<Recollide>,
) {
  match selection.into_inner() {
    None => {}
    Some(Selection { cube, face }) => {
      let source: DDD = *cube;
      let target_negative = *face;
      if mouse.just_pressed(MouseButton::Left) {
        pick_up_block(
          commands,
          player_inventory.as_mut(),
          source,
          &mut block_accessor,
          &mut rerender_inventory,
        );
        relight_events.send(RelightEvent::Relight(RelightType::BlockRemoved, source));
        recollide.0 = true;
      }
      if mouse.just_pressed(MouseButton::Right) {
        let block = player_inventory.items[hotbar_selection.0 as usize]
          .as_ref()
          .map(|x| x.block_or_item == BlockOrItem::Block(BlockId::LightEmitter))
          .unwrap_or(false);
        place_item_from_inventory(
          player_inventory.as_mut(),
          hotbar_selection.0 as usize,
          target_negative,
          &mut block_accessor,
          &rapier_context,
          &camera.single(),
        );
        if block {
          relight_events.send(RelightEvent::Relight(RelightType::LightSourceAdded, target_negative));
        } else {
          relight_events.send(RelightEvent::Relight(RelightType::BlockAdded, target_negative));
        }
        recollide.0 = true;
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
  if keys.just_pressed(KeyCode::Key4) {
    selected_hotbar.0 = 3;
  }
  if keys.just_pressed(KeyCode::Key5) {
    selected_hotbar.0 = 4;
  }
}
