use crate::ecs::components::blocks::{BlockSprite, DerefExt};
use crate::ecs::components::OverlayRender;
use crate::ecs::plugins::camera::{FPSCamera, Player, Recollide, Selection, SelectionRes};
use crate::ecs::plugins::game::ShikataganaiGameState;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::delta_to_side;
use crate::ecs::resources::player::{PlayerInventory, SelectedHotBar};
use crate::ecs::systems::user_interface::player_inventory::PlayerInventoryOpened;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseWheel;
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy_rapier3d::pipeline::QueryFilter;
use bevy_rapier3d::prelude::{Collider, InteractionGroups, RapierContext};
use bevy_rapier3d::rapier::prelude::Group;
use bevy_renet::renet::RenetClient;
use bincode::serialize;
use itertools::Itertools;
use iyes_loopless::prelude::NextState;
use num_traits::FloatConst;
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::{
  Block, BlockOrItem, BlockRotation, QuantifiedBlockOrItem, ReverseLocation,
};
use shikataganai_common::ecs::components::item::ItemId;
use shikataganai_common::ecs::resources::light::{LightLevel, RelightEvent};
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::networking::{ClientChannel, PlayerCommand};
use shikataganai_common::util::array::{sub_ddd, DDD};
use std::cmp::Ordering;
use std::ops::Deref;
use crate::ecs::plugins::rendering::particle_pipeline::{EffectSprite, Particle, ParticleEmitter};

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
            groups: Some(InteractionGroups::new(Group::GROUP_2, Group::GROUP_1)),
            exclude_collider: None,
            exclude_rigid_body: None,
            predicate: None,
          },
        )
        .is_none()
      {
        let mut phi = (camera.phi - f32::FRAC_PI_4()) % (f32::PI() * 2.0);
        if phi < 0.0 {
          phi += f32::PI() * 2.0;
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
        Some(*target_negative_block)
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

pub fn add_item_inventory(player_inventory: &mut PlayerInventory, item_id: ItemId, quant: u32) -> Option<()> {
  match player_inventory
    .items
    .iter_mut()
    .filter(|slot| {
      slot
        .as_ref()
        .map(|item| item.block_or_item == BlockOrItem::Item(item_id))
        .unwrap_or(true)
    })
    .sorted_by(|slot1, slot2| {
      if let Some(slot1) = slot1 && let Some(_slot2) = slot2 {
        if slot1.block_or_item == BlockOrItem::Item(item_id) {
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
        block_or_item: BlockOrItem::Item(item_id),
        quant: 0,
      })
    })
    .map(|slot| {
      slot.quant += quant;
      slot.quant == 1
    }) {
    None => {
      return None;
    }
    Some(true) => {}
    _ => {}
  }

  Some(())
}

fn pick_up_block(
  commands: &mut Commands,
  player_inventory: &mut PlayerInventory,
  coord: DDD,
  game_world: &mut GameWorld,
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
        if let Some(slot1) = slot1 && let Some(_slot2) = slot2 {
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
        // Pick up, do the rerender event
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

pub fn keyboard_input(
  mut commands: Commands,
  mut keyboard: EventReader<KeyboardInput>,
  player_inventory_opened: Option<Res<PlayerInventoryOpened>>,
  player_position: Query<&Transform, With<Player>>,
) {
  for KeyboardInput {
    scan_code: _scan_code,
    key_code,
    state,
  } in keyboard.iter()
  {
    if key_code == &Some(KeyCode::E) && state == &ButtonState::Pressed {
      if player_inventory_opened.is_some() {
        commands.remove_resource::<PlayerInventoryOpened>();
        commands.insert_resource(NextState(ShikataganaiGameState::Simulation));
      } else {
        commands.insert_resource(PlayerInventoryOpened);
        commands.insert_resource(NextState(ShikataganaiGameState::InterfaceOpened));
      }
    }
    if key_code == &Some(KeyCode::B) && state == &ButtonState::Pressed {
      commands.spawn(ParticleEmitter { location: player_position.single().translation, tile: EffectSprite::Smoke, lifetime: 100 });
    }
  }
}

pub fn action_input(
  mut commands: Commands,
  mouse: Res<Input<MouseButton>>,
  camera: Query<&FPSCamera>,
  selection: Res<SelectionRes>,
  mut player_inventory: ResMut<PlayerInventory>,
  hotbar_selection: Res<SelectedHotBar>,
  mut game_world: ResMut<GameWorld>,
  mut relight_events: EventWriter<RelightEvent>,
  rapier_context: Res<RapierContext>,
  mut recollide: ResMut<Recollide>,
  mut client: ResMut<RenetClient>,
  mut overlay_query: Query<&mut OverlayRender>,
) {
  match selection.into_inner().deref() {
    None => {}
    Some(Selection { cube, face }) => {
      let source: DDD = *cube;
      let target_negative = *face;
      if mouse.just_pressed(MouseButton::Left) {
        if let Some(()) = pick_up_block(&mut commands, player_inventory.as_mut(), source, &mut game_world) {
          client.send_message(
            ClientChannel::ClientCommand.id(),
            serialize(&PlayerCommand::BlockRemove { location: source }).unwrap(),
          );

          relight_events.send(RelightEvent::Relight(source));
          recollide.0 = true;
        }
      }
      if mouse.just_pressed(MouseButton::Right)
        && game_world
          .get(source)
          .and_then(|block| {
            block
              .deref_ext()
              .right_click_interface(block.entity, source, &mut commands, &mut client)
          })
          .is_none()
      {
        let block_copy = place_item_from_inventory(
          player_inventory.as_mut(),
          hotbar_selection.0 as usize,
          target_negative,
          &mut game_world,
          &rapier_context,
          camera.single(),
        );

        if let Some(block) = block_copy {
          client.send_message(
            ClientChannel::ClientCommand.id(),
            serialize(&PlayerCommand::BlockPlace {
              location: target_negative,
              block_transfer: block.into(),
            })
            .unwrap(),
          );
        } else {
          let block = game_world.get_mut(source).unwrap();

          let mut overlays = [BlockSprite::Empty; 6];
          if block.entity == Entity::from_bits(0) || overlay_query.get(block.entity).is_err() {
            overlays[delta_to_side(sub_ddd(target_negative, source))] = BlockSprite::Progress1;
            let e = commands
              .spawn((ReverseLocation(source), OverlayRender { overlays }))
              .id();
            block.entity = e;
          } else {
            let mut overlays = overlay_query.get_mut(block.entity).unwrap();
            overlays.overlays[delta_to_side(sub_ddd(target_negative, source))] = BlockSprite::Progress1;
          }
          // Do the crafting
          client.send_message(
            ClientChannel::ClientCommand.id(),
            serialize(&PlayerCommand::InitiateInWorldCraft { location: source }).unwrap(),
          );
        }

        game_world.set_light_level(target_negative, LightLevel::dark());

        relight_events.send(RelightEvent::Relight(target_negative));
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
    selected_hotbar.0 -= y.signum() as i32;
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
