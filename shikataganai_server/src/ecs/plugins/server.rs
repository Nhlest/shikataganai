use crate::ecs::resources::world::{send_chunk_data, ServerGameWorld};
use crate::ecs::systems::chunkgen::{collect_async_chunks, ChunkTask};
use crate::ecs::systems::light::relight_system;
use bevy::app::ScheduleRunnerSettings;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::utils::hashbrown::{HashMap, HashSet};
use bevy_renet::renet::{RenetError, RenetServer, ServerAuthentication, ServerConfig, ServerEvent};
use bevy_renet::RenetServerPlugin;
use bincode::*;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::BlockMeta;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::ecs::components::functors::InternalInventory;
use shikataganai_common::ecs::resources::light::{relight_helper, LightLevel, RelightEvent};
use shikataganai_common::ecs::resources::player::PlayerNickname;
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::networking::{
  server_connection_config, FunctorType, NetworkFrame, NetworkedEntities, PlayerCommand, PolarRotation, ServerChannel,
  ServerMessage, PROTOCOL_ID,
};
use shikataganai_common::util::array::{ImmediateNeighbours, DD, DDD};
use std::io::Write;
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};
use num_traits::float::FloatConst;

pub struct ShikataganaiServerPlugin;

#[derive(StageLabel)]
pub struct FixedUpdate;

#[derive(Default)]
pub struct ServerTick(u32);

#[derive(Default)]
pub struct PlayerEntities {
  pub players: HashMap<u64, Entity>,
}

#[derive(Default)]
pub struct UnAuthedPlayers {
  pub players: HashSet<u64>,
}

#[derive(Component)]
pub struct ClientId(u64);

pub struct ShikataganaiServerAddress {
  pub address: String,
}

impl Plugin for ShikataganaiServerPlugin {
  fn build(&self, app: &mut App) {
    let address = app.world.resource::<ShikataganaiServerAddress>().address.as_str();
    let server_addr = address.parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    println!("{}", server_addr);
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();

    // let on_fixed_step_simulation: SystemSet = ConditionSet::new()
    //   .into();
    // let on_fixed_step_simulation_stage = SystemStage::parallel().with_system_set(on_fixed_step_simulation);

    let server = RenetServer::new(
      current_time,
      ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure),
      server_connection_config(),
      socket,
    )
    .unwrap();

    app.add_event::<RelightEvent>();
    app.add_event::<FunctorRequestEvent>();

    app
      // .add_stage_after(
      //   CoreStage::PostUpdate,
      //   FixedUpdate,
      //   FixedTimestepStage::from_stage(Duration::from_millis(10), on_fixed_step_simulation_stage),
      // )
      .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(1.0 / 60.0)))
      .add_plugin(RenetServerPlugin { clear_events: false })
      .init_resource::<ServerTick>()
      .init_resource::<PlayerEntities>()
      .init_resource::<UnAuthedPlayers>()
      .insert_resource(server)
      .add_system(handle_events)
      .add_system(handle_functor_requests.after(handle_events))
      .add_system(sync_frame)
      .add_system(collect_async_chunks)
      .add_system(panic_handler)
      .add_system_to_stage(CoreStage::PostUpdate, relight_system);
  }
}

pub fn panic_handler(mut events: EventReader<RenetError>) {
  for i in events.iter() {
    println!("{}", i);
  }
}

#[derive(Debug)]
pub struct FunctorRequestEvent {
  pub client: u64,
  pub location: DDD,
  pub entity: Entity,
  pub functor_type: FunctorType,
}

pub fn handle_functor_requests(
  mut param_set: ParamSet<(&World, ResMut<RenetServer>)>,
  mut functor_events: EventReader<FunctorRequestEvent>,
) {
  for event in functor_events.iter() {
    let functor = serialize(match event.functor_type {
      FunctorType::InternalInventory => {
        if let Some(functor) = param_set.p0().get::<InternalInventory>(event.entity) {
          functor
        } else {
          return;
        }
      }
    })
    .unwrap();
    param_set.p1().send_message(
      event.client,
      ServerChannel::GameEvent.id(),
      serialize(&ServerMessage::Functor {
        location: event.location,
        functor_type: event.functor_type,
        functor,
      })
      .unwrap(),
    );
  }
}

pub fn handle_events(
  mut commands: Commands,
  mut server: ResMut<RenetServer>,
  mut server_events: EventReader<ServerEvent>,
  mut relight: EventWriter<RelightEvent>,
  mut functor_events: EventWriter<FunctorRequestEvent>,
  mut player_entities: ResMut<PlayerEntities>,
  mut unauthed_players: ResMut<UnAuthedPlayers>,
  mut query: Query<(Entity, &mut Transform, &mut PolarRotation, &PlayerNickname)>,
  mut game_world: ResMut<GameWorld>,
) {
  for event in server_events.iter() {
    match event {
      ServerEvent::ClientConnected(client_id, _) => {
        unauthed_players.players.insert(*client_id);
        println!("Client {} connected", client_id);
      }
      ServerEvent::ClientDisconnected(client_id) => {
        println!("Client {} disconnected", client_id);
        let entity = player_entities.players.remove(client_id).unwrap();
        // commands.entity(entity).despawn();
      }
    }
  }
  for client in server.clients_id().into_iter() {
    while let Some(message) = server.receive_message(client, 0) {
      let command: PlayerCommand = deserialize(&message).unwrap();
      match command {
        PlayerCommand::PlayerMove { translation } => {
          let player_entity = *player_entities.players.get(&client).unwrap();
          query.get_mut(player_entity).unwrap().1.translation = translation.0;
          *query.get_mut(player_entity).unwrap().2 = translation.1;
        }
        PlayerCommand::BlockRemove { location } => {
          if let Some(block) = game_world.get_mut(location) {
            *block = BlockId::Air.into();
            relight.send(RelightEvent::Relight(location));
            broadcast_but(server.as_mut(), client, ServerMessage::BlockRemove { location })
          }
        }
        PlayerCommand::BlockPlace { location, block_transfer } => {
          if let Some(block) = game_world.get_mut(location) {
            *block = block_transfer.into();
            if block.need_to_spawn_functors() {
              block.block.clone().spawn_or_add_functors(block, location, &mut commands);
            }
            relight.send(RelightEvent::Relight(location));
            game_world.set_light_level(location, LightLevel::dark());
            broadcast_but(server.as_mut(), client, ServerMessage::BlockPlace { location, block_transfer })
          }
        }
        PlayerCommand::RequestChunk { chunk_coord: coord } => {
          if let Some(chunk) = game_world.get_chunk_or_spawn(coord, &mut commands, client) {
            send_chunk_data(server.as_mut(), chunk, client);
          }
        }
        PlayerCommand::RequestFunctor { location, functor } => {
          if let Some(entity) = game_world.get(location).map(|block| block.entity) && entity != Entity::from_bits(0) {
            functor_events.send(FunctorRequestEvent {
              client,
              location,
              entity,
              functor_type: functor
            });
          }
        }
        PlayerCommand::PlayerAuth { nickname } => {
          if unauthed_players.players.contains(&client) {
            unauthed_players.players.remove(&client);
            let (player_entity, translation, rotation) = query.iter().find(|(_, _, _, player_nickname)| player_nickname.0 == nickname).map(|(entity, transform, rotation, _)| {
              (entity, transform.translation, *rotation)
            }).or_else(|| {
              let player_entity = commands
                .spawn()
                .insert(Transform::from_xyz(10.1, 45.0, 10.0))
                .insert(PolarRotation { phi: 0.0, theta: f32::FRAC_PI_2() })
                .insert(ClientId(client))
                .insert(PlayerNickname(nickname))
                .id();
              Some((player_entity, Vec3::new(10.1, 45.0, 10.0), PolarRotation { phi: 0.0, theta: f32::FRAC_PI_2() }))
            }).unwrap();

            if player_entities.players.iter().find(|(_, entity)| **entity == player_entity).is_some() {
              println!("Client taken!");
              continue;
            }

            for other_client in player_entities.players.keys() {
              let other_entity = *player_entities.players.get(other_client).unwrap();
              let (_, translation, rotation, _) = query.get(other_entity).unwrap();
              server.send_message(
                client,
                ServerChannel::GameEvent.id(),
                serialize(&ServerMessage::PlayerSpawn {
                  entity: other_entity,
                  id: *other_client,
                  translation: (translation.translation, *rotation),
                })
                .unwrap(),
              );
              server.send_message(
                *other_client,
                ServerChannel::GameEvent.id(),
                serialize(&ServerMessage::PlayerSpawn {
                  entity: player_entity,
                  id: client,
                  translation: (translation.translation, *rotation),
                }).unwrap(),
              );
            }
            server.send_message(
              client,
              ServerChannel::GameEvent.id(),
              serialize(&ServerMessage::AuthConfirmed {
                translation: (translation, rotation),
              }).unwrap(),
            );
            player_entities.players.insert(client, player_entity);
          }
        }
        PlayerCommand::AnimationStart { location, animation } => {
          for other_client in player_entities.players.keys() {
            if *other_client == client {
              continue;
            }
            server.send_message(*other_client, ServerChannel::GameEvent.id(), serialize(&ServerMessage::AnimationStart { location, animation: animation.clone() }).unwrap())
          }
        }
      }
    }
  }
}

pub fn broadcast_but(server: &mut RenetServer, client_exclude: u64, message: ServerMessage) {
  for broadcast_client in server.clients_id().into_iter() {
    if client_exclude != broadcast_client {
      server.send_message(
        broadcast_client,
        ServerChannel::GameEvent.id(),
        serialize(&message).unwrap(),
      );
    }
  }
}

pub fn sync_frame(
  mut server: ResMut<RenetServer>,
  mut tick: ResMut<ServerTick>,
  query: Query<(&ClientId, &Transform, &PolarRotation)>,
) {
  let mut players = vec![];
  let mut translations = vec![];
  query.iter().for_each(|(id, transform, rotation)| {
    players.push(id.0);
    translations.push((transform.translation, rotation.clone()))
  });
  let frame = NetworkFrame {
    tick: tick.0,
    entities: NetworkedEntities { players, translations },
  };
  tick.0 += 1;
  server.broadcast_message(ServerChannel::GameFrame.id(), serialize(&frame).unwrap())
}

pub fn get_chunk_coord(mut coord: DDD) -> DD {
  if coord.0 < 0 {
    coord.0 -= 15;
  }
  if coord.2 < 0 {
    coord.2 -= 15;
  }
  (coord.0 / 16, coord.2 / 16)
}
