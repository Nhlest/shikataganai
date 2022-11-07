use crate::ecs::components::blocks::animation::Animation;
use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::components::blocks::BlockMeta;
use crate::ecs::resources::light::LightLevel;
use crate::util::array::{DD, DDD};
use bevy::prelude::*;
use bevy_renet::renet::{ChannelConfig, ReliableChannelConfig, RenetConnectionConfig, UnreliableChannelConfig};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::Duration;

// -------------------------------------------------------------------------------------------
// -- ###  #    #   ###   ####   #####  #     #  #####  #    #  #####        #     #  ##### --
// --  #   #    #  #   #  #   #  #      ##   ##  #      #    #    #          ##   ##  #     --
// --  #   ##   #  #      #   #  #      # # # #  #      ##   #    #          # # # #  #     --
// --  #   # #  #  #      #   #  #      #  #  #  #      # #  #    #          #  #  #  #     --
// --  #   #  # #  #      ####   ####   #     #  ####   #  # #    #          #     #  ####  --
// --  #   #   ##  #      ##     #      #     #  #      #   ##    #          #     #  #     --
// --  #   #    #  #      # #    #      #     #  #      #    #    #          #     #  #     --
// --  #   #    #  #   #  #  #   #      #     #  #      #    #    #          #     #  #     --
// -- ###  #    #   ###   #   #  #####  #     #  #####  #    #    #          #     #  ##### --
// -------------------------------------------------------------------------------------------
pub const PROTOCOL_ID: u64 = 42;
pub const RELIABLE_CHANNEL_MAX_LENGTH: u64 = 10240;

pub enum ServerChannel {
  GameEvent,
  GameFrame,
}

impl ServerChannel {
  pub fn id(&self) -> u8 {
    match self {
      Self::GameEvent => 0,
      Self::GameFrame => 1,
    }
  }

  pub fn channels_config() -> Vec<ChannelConfig> {
    vec![
      ReliableChannelConfig {
        channel_id: Self::GameEvent.id(),
        message_resend_time: Duration::ZERO,
        max_message_size: RELIABLE_CHANNEL_MAX_LENGTH,
        packet_budget: RELIABLE_CHANNEL_MAX_LENGTH * 2,
        ..Default::default()
      }
      .into(),
      UnreliableChannelConfig {
        channel_id: Self::GameFrame.id(),
        message_send_queue_size: 2048,
        message_receive_queue_size: 2048,
        ..Default::default()
      }
      .into(),
    ]
  }
}

#[derive(Component, Debug, Deserialize, Serialize, Clone, Copy)]
pub struct PolarRotation {
  pub phi: f32,
  pub theta: f32,
}

type TranslationRotation = (Vec3, PolarRotation);

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessage {
  PlayerSpawn {
    entity: Entity,
    id: u64,
    translation: TranslationRotation,
  },
  AuthConfirmed {
    translation: TranslationRotation,
  },
  PlayerDespawn {
    id: u64,
  },
  BlockRemove {
    location: DDD,
  },
  BlockPlace {
    location: DDD,
    block_transfer: BlockTransfer,
  },
  ChunkData {
    chunk: Vec<u8>,
  },
  Relight {
    relights: Vec<(DDD, LightLevel)>,
  },
  // TODO: figure out rare cases when these commands happen right after client updates block state (chops down / puts a new one)
  Functor {
    location: DDD,
    functor_type: FunctorType,
    functor: Vec<u8>,
  },
  AnimationStart {
    location: DDD,
    animation: Animation,
  },
}

impl Display for ServerMessage {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ServerMessage::PlayerSpawn { .. } => f.write_str("PlayerSpawn"),
      ServerMessage::AuthConfirmed { .. } => f.write_str("AuthConfirmed"),
      ServerMessage::PlayerDespawn { .. } => f.write_str("PlayerDespawn"),
      ServerMessage::BlockRemove { .. } => f.write_str("BlockRemove"),
      ServerMessage::BlockPlace { .. } => f.write_str("BlockPlace"),
      ServerMessage::ChunkData { .. } => f.write_str("ChunkData"),
      ServerMessage::Relight { .. } => f.write_str("Relight"),
      ServerMessage::Functor { .. } => f.write_str("Functor"),
      ServerMessage::AnimationStart { .. } => f.write_str("AnimationStart"),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
  pub players: Vec<u64>,
  pub translations: Vec<TranslationRotation>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkFrame {
  pub tick: u32,
  pub entities: NetworkedEntities,
}

pub enum ClientChannel {
  ClientCommand,
}

impl ClientChannel {
  pub fn id(&self) -> u8 {
    match self {
      Self::ClientCommand => 0,
    }
  }

  pub fn channels_config() -> Vec<ChannelConfig> {
    vec![ReliableChannelConfig {
      channel_id: Self::ClientCommand.id(),
      message_resend_time: Duration::ZERO,
      ..Default::default()
    }
    .into()]
  }
}

pub fn server_connection_config() -> RenetConnectionConfig {
  RenetConnectionConfig {
    send_channels_config: ServerChannel::channels_config(),
    receive_channels_config: ClientChannel::channels_config(),
    ..Default::default()
  }
}

pub fn client_connection_config() -> RenetConnectionConfig {
  RenetConnectionConfig {
    send_channels_config: ClientChannel::channels_config(),
    receive_channels_config: ServerChannel::channels_config(),
    ..Default::default()
  }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct BlockTransfer {
  pub block: BlockId,
  pub meta: BlockMeta,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum FunctorType {
  InternalInventory,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerCommand {
  PlayerAuth {
    nickname: String,
  },
  PlayerMove {
    translation: TranslationRotation,
  },
  BlockRemove {
    location: DDD,
  },
  BlockPlace {
    location: DDD,
    block_transfer: BlockTransfer,
  },
  RequestChunk {
    chunk_coord: DD,
  },
  RequestFunctor {
    location: DDD,
    functor: FunctorType,
  },
  AnimationStart {
    location: DDD,
    animation: Animation,
  },
}
