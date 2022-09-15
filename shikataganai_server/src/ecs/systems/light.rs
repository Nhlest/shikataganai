use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use bincode::serialize;
use shikataganai_common::ecs::resources::light::{relight_helper, RelightEvent};
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::networking::{ServerChannel, ServerMessage};

pub fn relight_system(
  mut relight: EventReader<RelightEvent>,
  mut game_world: ResMut<GameWorld>,
  mut server: ResMut<RenetServer>,
) {
  let mut relights = vec![];
  for coord in relight_helper(&mut relight, game_world.as_mut()).iter() {
    relights.push((*coord, game_world.get_light_level(*coord).unwrap()))
  }
  if !relights.is_empty() {
    let message = serialize(&ServerMessage::Relight { relights }).unwrap();
    // TODO: idk
    if message.len() < 2000 {
      server.broadcast_message(ServerChannel::GameEvent.id(), message)
    }
  }
}
