use crate::ecs::components::blocks::{animate, AnimationTrait, ChestAnimations, DerefExt};
use crate::ecs::plugins::camera::{FPSCamera, Recollide, Selection};
use crate::ecs::resources::player::{PlayerInventory, RerenderInventory, SelectedHotBar};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::prelude::{Collider, InteractionGroups, RapierContext};
use bevy_renet::renet::RenetClient;
use bincode::serialize;
use itertools::Itertools;
use num_traits::FloatConst;
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::{Block, BlockOrItem, BlockRotation, QuantifiedBlockOrItem};
use shikataganai_common::ecs::resources::light::{LightLevel, RelightEvent};
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::networking::{ClientChannel, PlayerCommand};
use shikataganai_common::util::array::DDD;
use std::cmp::Ordering;
use iyes_loopless::state::NextState;
use crate::ecs::plugins::game::ShikataganaiGameState;

fn place_item_from_inventory(
  player_inventory: &mut PlayerInventory,
  item_idx: usize,
  coord: DDD,
  game_world: &mut GameWorld,
  rapier_context: &RapierContext,
  camera: &FPSCamera,
) -> Option<Block> {
  if let Some(Some(QuantifiedBlockOrItem {
    block_or_item: BlockOrItem::Block(block),
    quant,
  })) = player_inventory.items.get_mut(item_idx)
  {
    if let Some(target_negative_block) = game_world.get_mut(coord) {
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
        Some(target_negative_block.clone())
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  }
}

fn pick_up_block(
  commands: &mut Commands,
  player_inventory: &mut PlayerInventory,
  coord: DDD,
  game_world: &mut GameWorld,
  rerender_inventory: &mut ResMut<RerenderInventory>,
) -> Option<()> {
  if let Some(source_block) = game_world.get_mut(coord) {
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
      .sorted_by(|slot1, slot2| {
        if let Some(slot1) = slot1 && let Some(slot2) = slot2 {
          if slot1.block_or_item == BlockOrItem::Block(block) {
            Ordering::Greater
          } else {
            Ordering::Less
          }
        } else if slot1.is_some() {
          Ordering::Less
        } else {
          Ordering::Greater
        }
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
        return None;
      }
      Some(true) => {
        // New item added - rerender required
        rerender_inventory.0 = true;
      }
      _ => {}
    }

    if source_block.entity != Entity::from_bits(0) {
      commands.entity(source_block.entity).despawn_recursive();
      source_block.entity = Entity::from_bits(0);
    }
    Some(())
  } else {
    None
  }
}

pub fn action_input(
  mut commands: Commands,
  mouse: Res<Input<MouseButton>>,
  camera: Query<&FPSCamera>,
  selection: Res<Option<Selection>>,
  mut player_inventory: ResMut<PlayerInventory>,
  hotbar_selection: Res<SelectedHotBar>,
  mut game_world: ResMut<GameWorld>,
  mut relight_events: EventWriter<RelightEvent>,
  rapier_context: Res<RapierContext>,
  mut rerender_inventory: ResMut<RerenderInventory>,
  mut recollide: ResMut<Recollide>,
  mut client: ResMut<RenetClient>,
  mut windows: ResMut<Windows>,
) {
  let window = windows.get_primary_mut().unwrap();
  match selection.into_inner() {
    None => {}
    Some(Selection { cube, face }) => {
      let source: DDD = *cube;
      let target_negative = *face;
      if mouse.just_pressed(MouseButton::Left) {
        if let Some(()) = pick_up_block(
          &mut commands,
          player_inventory.as_mut(),
          source,
          &mut game_world,
          &mut rerender_inventory,
        ) {
          client.send_message(
            ClientChannel::ClientCommand.id(),
            serialize(&PlayerCommand::BlockRemove { location: source }).unwrap(),
          );

          relight_events.send(RelightEvent::Relight(source));
          recollide.0 = true;
        }
      }
      if mouse.just_pressed(MouseButton::Right) {
        if game_world
          .get(source)
          .map(|block| block.deref_ext().right_click_interface(block.entity, &mut commands))
          .flatten()
          .is_none()
        {
          let block_copy = place_item_from_inventory(
            player_inventory.as_mut(),
            hotbar_selection.0 as usize,
            target_negative,
            &mut game_world,
            &rapier_context,
            &camera.single(),
          );

          block_copy.map(|block| {
            client.send_message(
              ClientChannel::ClientCommand.id(),
              serialize(&PlayerCommand::BlockPlace {
                location: target_negative,
                block_transfer: block.into(),
              })
              .unwrap(),
            );
          });

          game_world.set_light_level(target_negative, LightLevel::dark());

          relight_events.send(RelightEvent::Relight(target_negative));
          recollide.0 = true;
        } else {
          commands.insert_resource(NextState(ShikataganaiGameState::InterfaceOpened));
          window.set_cursor_lock_mode(false);
          window.set_cursor_visibility(true);
          game_world.get(source).map(|block| {
            if block.block == BlockId::Chest {
              animate(&mut commands, block.entity, ChestAnimations::Open.get_animation());
              client.send_message(ClientChannel::ClientCommand.id(), serialize(&PlayerCommand::AnimationStart { location: source, animation: ChestAnimations::Open.get_animation() }).unwrap());
            }
          });
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
  if keys.just_pressed(KeyCode::Key4) {
    selected_hotbar.0 = 3;
  }
  if keys.just_pressed(KeyCode::Key5) {
    selected_hotbar.0 = 4;
  }
}
