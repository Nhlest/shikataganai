use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::{ClientAuthentication, RenetClient, RenetError};
use bevy_renet::RenetClientPlugin;
use bincode::*;
use iyes_loopless::prelude::ConditionSet;
use num_traits::{Float, FloatConst};
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::networking::{
  client_connection_config, ClientChannel, NetworkFrame, PlayerCommand, PolarRotation, ServerChannel, ServerMessage,
  PROTOCOL_ID,
};
use std::net::UdpSocket;
use std::time::SystemTime;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::util::array::{Bounds, DD};

use crate::ecs::plugins::camera::{FPSCamera, Player};
use crate::ecs::plugins::game::in_game;
use crate::ecs::plugins::rendering::mesh_pipeline::loader::{get_mesh_from_storage, GltfMeshStorageHandle, Meshes};
use crate::ecs::plugins::rendering::mesh_pipeline::AmongerTextureHandle;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::{RelightEvent, RelightType, RemeshEvent};
use crate::ecs::resources::chunk_map::{BlockAccessor, ChunkMap};
use crate::ecs::resources::chunk_map::BlockAccessorStatic;
use crate::GltfMeshStorage;

#[derive(Default)]
struct NetworkMapping(HashMap<Entity, Entity>);

#[derive(Debug)]
struct PlayerInfo {
  client_entity: Entity,
  server_entity: Entity,
}

#[derive(Debug, Default)]
struct ClientLobby {
  players: HashMap<u64, PlayerInfo>,
}

#[derive(Component)]
pub struct AmongerSkeleton {
  visor: Entity,
  legl: Entity,
  legr: Entity,
}

#[derive(Component)]
pub struct LegAnimationFrame(f32, u32);

pub struct ShikataganaiClientPlugin;

impl Plugin for ShikataganaiClientPlugin {
  fn build(&self, app: &mut App) {
    let on_game_simulation_continuous = ConditionSet::new()
      .run_if(in_game)
      .with_system(send_system)
      .with_system(receive_system)
      .into();

    app
      .add_plugin(RenetClientPlugin { clear_events: false })
      .init_resource::<ClientLobby>()
      .init_resource::<NetworkMapping>()
      .add_system(panic_handler)
      .add_system_set(on_game_simulation_continuous);
  }
}

fn panic_handler(mut events: EventReader<RenetError>) {
  for i in events.iter() {
    println!("{}", i);
  }
}

fn spawn_amonger(
  commands: &mut Commands,
  mesh_storage: &Assets<GltfMeshStorage>,
  mesh_storage_handle: &GltfMeshStorageHandle,
  translation: Vec3,
  amonger_texture: &AmongerTextureHandle,
) -> Entity {
  let (client_entity, legl, legr, visor) = commands
    .spawn()
    .insert(Transform::from_translation(translation))
    .insert(GlobalTransform::default())
    .add_children(|c| {
      let body = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerBody);
      let legl = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerLegL);
      let legr = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerLegR);
      let backpack = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerBackpack);
      let visor = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerVisor);
      c.spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_translation(body.1))
        .insert(body.0.clone())
        .insert(amonger_texture.0.clone());
      let legl = c
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_translation(legl.1))
        .insert(legl.0.clone())
        .insert(amonger_texture.0.clone())
        .id();
      let legr = c
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_translation(legr.1))
        .insert(legr.0.clone())
        .insert(amonger_texture.0.clone())
        .id();
      c.spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_translation(backpack.1))
        .insert(backpack.0.clone())
        .insert(amonger_texture.0.clone());
      let visor = c
        .spawn()
        .insert(GlobalTransform::default())
        .insert(Transform::from_translation(visor.1))
        .insert(visor.0.clone())
        .insert(amonger_texture.0.clone())
        .id();
      (c.parent_entity(), legl, legr, visor)
    });
  commands
    .entity(client_entity)
    .insert(AmongerSkeleton { visor, legl, legr });
  commands.entity(client_entity).insert(LegAnimationFrame(0.0, 0));
  client_entity
}

fn receive_system(
  mut commands: Commands,
  mut client: ResMut<RenetClient>,
  mut lobby: ResMut<ClientLobby>,
  mut network_mapping: ResMut<NetworkMapping>,
  mut query: Query<&mut Transform>,
  mut query_leg_animation: Query<&mut LegAnimationFrame>,
  query_skeleton: Query<&AmongerSkeleton>,
  mesh_storage: Res<Assets<GltfMeshStorage>>,
  mesh_storage_handle: Res<GltfMeshStorageHandle>,
  amonger_texture: Res<AmongerTextureHandle>,
  mut block_accessor: BlockAccessorStatic,
  mut relight: EventWriter<RelightEvent>,
  time: Res<Time>,
  mut remesh: EventWriter<RemeshEvent>
) {
  let client_id = client.client_id();
  while let Some(message) = client.receive_message(ServerChannel::ChunkTransfer.id()) {
    let chunk : Chunk = deserialize(&message).unwrap();
    // dbg!(&message);
    // let chunk : DD = deserialize(&message).unwrap();
    // println!("Received {:?}", chunk);
    let dd = ChunkMap::get_chunk_coord(chunk.grid.bounds.0);
    println!("Received {:?}", dd);
    let chunk_entity = commands.spawn().insert(chunk).id();
    block_accessor.chunk_map.map.get_mut(&dd).unwrap().entity = Some(chunk_entity);

    for i in dd.0 - 1..=dd.0 + 1 {
      for j in dd.1 - 1..=dd.1 + 1 {
        remesh.send(RemeshEvent::Remesh((i, j)));
      }
    }
  }

  while let Some(message) = client.receive_message(ServerChannel::GameEvent.id()) {
    let server_message: ServerMessage = deserialize(&message).unwrap();
    match server_message {
      ServerMessage::PlayerSpawn {
        entity,
        id,
        translation,
      } => {
        if client_id == id {
          continue;
        }
        let client_entity = spawn_amonger(
          &mut commands,
          mesh_storage.as_ref(),
          &mesh_storage_handle,
          translation.0,
          amonger_texture.as_ref(),
        );
        lobby.players.insert(
          id,
          PlayerInfo {
            client_entity,
            server_entity: entity,
          },
        );
        network_mapping.0.insert(entity, client_entity);
      }
      ServerMessage::PlayerDespawn { id } => {
        let client_entity = lobby.players.get(&id).unwrap().client_entity;
        commands.entity(client_entity).despawn_recursive();
      }
      ServerMessage::BlockRemove { location } => {
        block_accessor.get_mut(location).map(|b| b.block = BlockId::Air);
        relight.send(RelightEvent::Relight(RelightType::BlockRemoved, location));
      }
      ServerMessage::BlockPlace { location, block } => {
        block_accessor.get_mut(location).map(|b| *b = block);
        relight.send(RelightEvent::Relight(RelightType::BlockAdded, location));
      }
    }
  }

  while let Some(message) = client.receive_message(ServerChannel::GameFrame.id()) {
    let server_message: NetworkFrame = deserialize(&message).unwrap();
    for (id, translation) in server_message
      .entities
      .players
      .iter()
      .zip(server_message.entities.translations.iter())
    {
      if client_id == *id {
        continue;
      }
      if let Some(entity) = lobby.players.get(id).map(|e| e.client_entity) {
        if query.get(entity).is_err() {
          continue;
        }
        let current_location = query.get_mut(entity).unwrap().translation;
        let leg_animation = query_leg_animation.get_mut(entity).unwrap().into_inner();

        if current_location.distance(translation.0) <= f32::epsilon() * 10.0 {
          leg_animation.1 += 1;
        } else {
          leg_animation.1 = 0;
        }

        if leg_animation.1 > 20 {
          leg_animation.0 *= 0.5;
        } else {
          leg_animation.0 += time.delta().as_secs_f32() * 3.0;
          if leg_animation.0 >= 2.0 {
            leg_animation.0 = 0.0;
          }
        }
        if let Ok(skeleton) = query_skeleton.get(entity) {
          let PolarRotation { phi, theta } = translation.1;
          let _ = query.get_mut(entity).map(|mut transform| {
            transform.translation = translation.0;
            transform.rotation = Quat::from_rotation_y(-phi);
          });
          let _ = query.get_mut(skeleton.visor).map(|mut transform| {
            transform.rotation = Quat::from_rotation_z(-theta + f32::FRAC_PI_2());
          });
          let _ = query.get_mut(skeleton.legl).map(|mut transform| {
            if leg_animation.0 > 0.0 && leg_animation.0 <= 0.5 {
              transform.rotation = Quat::from_rotation_z(leg_animation.0 * f32::PI());
            }
            if leg_animation.0 > 0.5 && leg_animation.0 <= 1.0 {
              transform.rotation = Quat::from_rotation_z((1.0 - leg_animation.0) * f32::PI());
            }
            if leg_animation.0 > 1.0 && leg_animation.0 <= 1.5 {
              transform.rotation = Quat::from_rotation_z(-(leg_animation.0 - 1.0) * f32::PI());
            }
            if leg_animation.0 > 1.5 && leg_animation.0 <= 2.0 {
              transform.rotation = Quat::from_rotation_z(-(2.0 - leg_animation.0) * f32::PI());
            }
          });
          let _ = query.get_mut(skeleton.legr).map(|mut transform| {
            if leg_animation.0 > 0.0 && leg_animation.0 <= 0.5 {
              transform.rotation = Quat::from_rotation_z(-leg_animation.0 * f32::PI());
            }
            if leg_animation.0 > 0.5 && leg_animation.0 <= 1.0 {
              transform.rotation = Quat::from_rotation_z(-(1.0 - leg_animation.0) * f32::PI());
            }
            if leg_animation.0 > 1.0 && leg_animation.0 <= 1.5 {
              transform.rotation = Quat::from_rotation_z((leg_animation.0 - 1.0) * f32::PI());
            }
            if leg_animation.0 > 1.5 && leg_animation.0 <= 2.0 {
              transform.rotation = Quat::from_rotation_z((2.0 - leg_animation.0) * f32::PI());
            }
          });
        }
      }
    }
  }
}

fn send_system(
  mut client: ResMut<RenetClient>,
  query_player: Query<&Transform, With<Player>>,
  query_camera: Query<&FPSCamera>,
) {
  let translation = query_player.single().translation;
  let rotation = query_camera.single();
  client.send_message(
    ClientChannel::ClientCommand.id(),
    serialize(&PlayerCommand::PlayerMove {
      translation: (
        Vec3::new(translation.x, translation.y - 1.5, translation.z),
        PolarRotation {
          phi: rotation.phi,
          theta: rotation.theta,
        },
      ),
    })
    .unwrap(),
  );
}

pub fn spawn_client(mut commands: Commands, address: String) {
  let server_addr = address.parse().unwrap();
  let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
  let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
  let client_id = current_time.as_millis() as u64;

  let client = RenetClient::new(
    current_time,
    socket,
    client_connection_config(),
    ClientAuthentication::Unsecure {
      protocol_id: PROTOCOL_ID,
      client_id,
      server_addr,
      user_data: None,
    },
  )
  .unwrap();

  commands.insert_resource(client);
}
