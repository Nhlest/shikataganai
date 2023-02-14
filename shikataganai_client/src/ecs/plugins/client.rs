use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::{ClientAuthentication, RenetClient, RenetError};
use bevy_renet::RenetClientPlugin;
use bincode::*;
use flate2::read::ZlibDecoder;
use iyes_loopless::prelude::ConditionSet;
use num_traits::{Float, FloatConst};
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::Block;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::ecs::components::functors::InternalInventory;
use shikataganai_common::ecs::resources::light::RelightEvent;
use shikataganai_common::ecs::resources::player::PlayerNickname;
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::networking::{
  client_connection_config, ClientChannel, FunctorType, NetworkFrame, PlayerCommand, PolarRotation, ServerChannel,
  ServerMessage, PROTOCOL_ID,
};
use std::io::Read;
use std::net::UdpSocket;
use std::time::SystemTime;
use tracing::Level;

use crate::ecs::components::blocks::animate;
use crate::ecs::plugins::camera::{FPSCamera, Player, Recollide};
use crate::ecs::plugins::console::ConsoleText;
use crate::ecs::plugins::game::{in_game, LocalTick};
use crate::ecs::plugins::rendering::mesh_pipeline::loader::{get_mesh_from_storage, GltfMeshStorageHandle, Meshes};
use crate::ecs::plugins::rendering::mesh_pipeline::systems::MeshMarker;
use crate::ecs::plugins::rendering::mesh_pipeline::AmongerTextureHandle;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::RemeshEvent;
use crate::ecs::resources::player::PlayerInventory;
use crate::ecs::systems::input::add_item_inventory;
use crate::GltfMeshStorage;

#[derive(Default, Resource)]
struct NetworkMapping(HashMap<Entity, Entity>);

#[derive(Debug, Resource)]
struct PlayerInfo {
  client_entity: Entity,
}

#[derive(Debug, Default, Resource)]
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

#[derive(Component)]
pub struct Requested;

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

pub fn send_message(client: &mut RenetClient, message: PlayerCommand) {
  client.send_message(ClientChannel::ClientCommand.id(), serialize(&message).unwrap());
}

fn spawn_amonger(
  commands: &mut Commands,
  mesh_storage: &Assets<GltfMeshStorage>,
  mesh_storage_handle: &GltfMeshStorageHandle,
  translation: Vec3,
  amonger_texture: &AmongerTextureHandle,
) -> Entity {
  let (client_entity, legl, legr, visor) = commands
    .spawn((Transform::from_translation(translation), GlobalTransform::default()))
    .add_children(|c| {
      let body = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerBody);
      let legl = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerLegL);
      let legr = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerLegR);
      let backpack = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerBackpack);
      let visor = get_mesh_from_storage(mesh_storage_handle, mesh_storage, Meshes::AmongerVisor);
      c.spawn((
        GlobalTransform::default(),
        Transform::from_translation(body.1),
        body.0.clone(),
        MeshMarker,
        amonger_texture.0.clone(),
      ));
      let legl = c
        .spawn((
          GlobalTransform::default(),
          Transform::from_translation(legl.1),
          legl.0.clone(),
          MeshMarker,
          amonger_texture.0.clone(),
        ))
        .id();
      let legr = c
        .spawn((
          GlobalTransform::default(),
          Transform::from_translation(legr.1),
          legr.0.clone(),
          MeshMarker,
          amonger_texture.0.clone(),
        ))
        .id();
      c.spawn((
        GlobalTransform::default(),
        Transform::from_translation(backpack.1),
        backpack.0.clone(),
        MeshMarker,
        amonger_texture.0.clone(),
      ));
      let visor = c
        .spawn((
          GlobalTransform::default(),
          Transform::from_translation(visor.1),
          visor.0.clone(),
          MeshMarker,
          amonger_texture.0.clone(),
        ))
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
  mut relight: EventWriter<RelightEvent>,
  mut remesh: EventWriter<RemeshEvent>,
  (mut network_mapping, mut game_world, mut recollide, mut client, mut lobby, mut player_inventory): (
    ResMut<NetworkMapping>,
    ResMut<GameWorld>,
    ResMut<Recollide>,
    ResMut<RenetClient>,
    ResMut<ClientLobby>,
    ResMut<PlayerInventory>,
  ),
  mesh_storage_handle: Res<GltfMeshStorageHandle>,
  amonger_texture: Res<AmongerTextureHandle>,
  mesh_storage: Res<Assets<GltfMeshStorage>>,
  player_nickname: Res<PlayerNickname>,
  time: Res<Time>,
  mut query_leg_animation: Query<&mut LegAnimationFrame>,
  query_skeleton: Query<&AmongerSkeleton>,
  mut player_entity: Query<Entity, With<Player>>,
  mut fps_camera_query: Query<&mut FPSCamera>,
  mut query: Query<&mut Transform>,
  mut event_writer: EventWriter<ConsoleText>,
  tick: Res<LocalTick>,
) {
  let client_id = client.client_id();

  while let Some(message) = client.receive_message(ServerChannel::GameEvent.id()) {
    let server_message: ServerMessage = deserialize(&message).unwrap();
    event_writer.send(ConsoleText {
      text: format!("{}", &server_message),
      level: Level::DEBUG,
      age: **tick,
    });
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
        lobby.players.insert(id, PlayerInfo { client_entity });
        network_mapping.0.insert(entity, client_entity);
      }
      ServerMessage::PlayerDespawn { id } => {
        let client_entity = lobby.players.get(&id).unwrap().client_entity;
        commands.entity(client_entity).despawn_recursive();
      }
      ServerMessage::BlockRemove { location } => {
        game_world.get_mut(location).map(|b| {
          b.block = BlockId::Air;
          if b.entity != Entity::from_bits(0) {
            commands.entity(b.entity).despawn_recursive();
            b.entity = Entity::from_bits(0);
            recollide.0 = true;
          }
        });
        remesh.send(RemeshEvent::Remesh(GameWorld::get_chunk_coord(location)));
      }
      ServerMessage::BlockPlace {
        location,
        block_transfer,
      } => {
        game_world.get_mut(location).map(|b| *b = block_transfer.into());
        remesh.send(RemeshEvent::Remesh(GameWorld::get_chunk_coord(location)));
        recollide.0 = true;
      }
      ServerMessage::ChunkData { chunk } => {
        let mut decoder = ZlibDecoder::new(chunk.as_slice());
        let mut message = Vec::new();
        decoder.read_to_end(&mut message).unwrap();
        let mut chunk: Chunk = deserialize(&message).unwrap();
        chunk.grid.map_in_place(|_, block| Block {
          entity: Entity::from_bits(0),
          ..*block
        });
        let chunk_coord = GameWorld::get_chunk_coord(chunk.grid.bounds.0);
        game_world.chunks.insert(chunk_coord, chunk);
        game_world.remove_from_generating(chunk_coord);

        for i in chunk_coord.0 - 1..=chunk_coord.0 + 1 {
          for j in chunk_coord.1 - 1..=chunk_coord.1 + 1 {
            remesh.send(RemeshEvent::Remesh((i, j)));
          }
        }
      }
      ServerMessage::Relight { relights } => {
        for (coord, light) in relights {
          game_world.set_light_level(coord, light);
          relight.send(RelightEvent::Relight(coord));
        }
      }
      ServerMessage::Functor {
        location,
        functor_type,
        functor,
      } => {
        if let Some(block) = game_world.get_mut(location) {
          let mut commands = if block.entity == Entity::from_bits(0) {
            commands.spawn_empty()
          } else {
            commands.entity(block.entity)
          };

          match functor_type {
            FunctorType::InternalInventory => {
              let functor: InternalInventory = deserialize(&functor).unwrap();
              commands.insert(functor);
            }
          }
          commands.remove::<Requested>();
          block.entity = commands.id();
        }
      }
      ServerMessage::AuthConfirmed {
        translation: (translation, rotation),
      } => {
        let entity = player_entity.single_mut();
        let mut fps_camera = fps_camera_query.single_mut();
        let mut transform = query.get_mut(entity).unwrap();
        commands.entity(entity).insert(player_nickname.as_ref().clone());
        fps_camera.phi = rotation.phi;
        fps_camera.theta = rotation.theta;
        transform.translation = translation + Vec3::new(0.0, 1.8, 0.0); // TODO: figure this out, player spawns below actual position
        recollide.0 = true;
      }
      ServerMessage::AnimationStart { location, animation } => {
        if let Some(entity) = game_world.get(location).map(|block| block.entity) && entity != Entity::from_bits(0) {
          animate(&mut commands, entity, animation);
        }
      }
      ServerMessage::ItemAdd { item, quant } => {
        add_item_inventory(player_inventory.as_mut(), item, quant);
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
  query_player: Query<&Transform, (With<Player>, With<PlayerNickname>)>,
  query_camera: Query<&FPSCamera>,
) {
  if let Some(translation) = query_player.iter().next().map(|transform| transform.translation) && let Some(rotation) = query_camera.iter().next() {
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
}

pub fn spawn_client(commands: &mut Commands, _player_entity: Entity, address: String, nickname: String) {
  let server_addr = address.parse().unwrap();
  let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
  let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
  let client_id = current_time.as_millis() as u64;
  commands.insert_resource(PlayerNickname(nickname));

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
