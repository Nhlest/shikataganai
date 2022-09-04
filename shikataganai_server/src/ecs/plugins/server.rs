use crate::ecs::systems::chunkgen::{collect_async_chunks, ChunkTask};
use bevy::app::ScheduleRunnerSettings;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::{RenetError, RenetServer, ServerAuthentication, ServerConfig, ServerEvent};
use bevy_renet::RenetServerPlugin;
use bincode::*;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::networking::{
  server_connection_config, NetworkFrame, NetworkedEntities, PlayerCommand, PolarRotation, ServerChannel,
  ServerMessage, PROTOCOL_ID,
};
use std::net::UdpSocket;
use std::time::{Duration, SystemTime};

pub struct ShikataganaiServerPlugin;

#[derive(StageLabel)]
pub struct FixedUpdate;

#[derive(Default)]
pub struct ServerTick(u32);

#[derive(Default)]
pub struct PlayerEntities {
  pub players: HashMap<u64, Entity>,
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
      .insert_resource(server)
      .add_system(handle_events)
      .add_system(sync_frame)
      .add_system(collect_async_chunks)
      .add_system(panic_handler);
  }
}

pub fn panic_handler(mut events: EventReader<RenetError>) {
  for i in events.iter() {
    println!("{}", i);
  }
}

pub fn handle_events(
  mut commands: Commands,
  mut server: ResMut<RenetServer>,
  mut server_events: EventReader<ServerEvent>,
  mut player_entities: ResMut<PlayerEntities>,
  mut query: Query<(&mut Transform, &mut PolarRotation)>,
) {
  for event in server_events.iter() {
    match event {
      ServerEvent::ClientConnected(client_id, _) => {
        let player_entity = commands
          .spawn()
          .insert(Transform::from_xyz(10.1, 45.0, 10.0))
          .insert(PolarRotation { phi: 0.0, theta: 0.0 })
          .insert(ClientId(*client_id))
          .id();
        for other_client in player_entities.players.keys() {
          let other_entity = *player_entities.players.get(other_client).unwrap();
          let (translation, rotation) = query.get(other_entity).unwrap();
          server.send_message(
            *client_id,
            ServerChannel::GameEvent.id(),
            serialize(&ServerMessage::PlayerSpawn {
              entity: other_entity,
              id: *other_client,
              translation: (translation.translation, rotation.clone()),
            })
            .unwrap(),
          );
        }
        player_entities.players.insert(*client_id, player_entity);
        server.broadcast_message(
          ServerChannel::GameEvent.id(),
          serialize(&ServerMessage::PlayerSpawn {
            entity: player_entity,
            id: *client_id,
            translation: (Vec3::new(10.1, 45.0, 10.0), PolarRotation { phi: 0.0, theta: 0.0 }),
          })
          .unwrap(),
        );
        println!("Client {} connected", client_id);
      }
      ServerEvent::ClientDisconnected(client_id) => {
        let entity = player_entities.players.remove(client_id).unwrap();
        commands.entity(entity).despawn();
      }
    }
  }
  for client in server.clients_id().into_iter() {
    while let Some(message) = server.receive_message(client, 0) {
      let command: PlayerCommand = deserialize(&message).unwrap();
      match command {
        PlayerCommand::PlayerMove { translation } => {
          let player_entity = *player_entities.players.get(&client).unwrap();
          query.get_mut(player_entity).unwrap().0.translation = translation.0;
          *query.get_mut(player_entity).unwrap().1 = translation.1;
        }
        PlayerCommand::BlockRemove { location } => {
          for broadcast_client in server.clients_id().into_iter() {
            if client != broadcast_client {
              server.send_message(
                broadcast_client,
                ServerChannel::GameEvent.id(),
                serialize(&ServerMessage::BlockRemove { location }).unwrap(),
              );
            }
          }
        }
        PlayerCommand::BlockPlace { location, block } => {
          for broadcast_client in server.clients_id().into_iter() {
            if client != broadcast_client {
              server.send_message(
                broadcast_client,
                ServerChannel::GameEvent.id(),
                serialize(&ServerMessage::BlockPlace {
                  location,
                  block: block.clone(),
                })
                .unwrap(),
              );
            }
          }
        }
        PlayerCommand::RequestChunk { coord } => {
          let dispatcher = AsyncComputeTaskPool::get();
          commands.spawn().insert(ChunkTask {
            task: dispatcher.spawn(Chunk::generate(coord)),
            coord,
            client,
          });
        }
      }
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
