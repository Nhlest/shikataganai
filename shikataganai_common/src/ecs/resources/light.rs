use crate::ecs::resources::world::GameWorld;
use crate::util::array::{ImmediateNeighbours, DDD};
use bevy::prelude::*;
use bevy::render::render_resource::encase::internal::{BufferMut, WriteInto, Writer};
use bevy::render::render_resource::encase::private::Metadata;
use bevy::render::render_resource::ShaderType;
use bevy::utils::HashSet;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub enum RelightEvent {
  Relight(DDD),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Component)]
pub struct LightLevel {
  pub heaven: u8,
  pub hearth: u8,
  pub light_source: u8,
}

impl ShaderType for LightLevel {
  type ExtraMetadata = ();
  const METADATA: Metadata<()> = Metadata::from_alignment_and_size(8, 8);
}

impl WriteInto for LightLevel {
  fn write_into<B>(&self, writer: &mut Writer<B>)
  where
    B: BufferMut,
  {
    writer.write(&[self.hearth, 0, 0, 0, self.heaven, 0, 0, 0])
  }
}

impl LightLevel {
  pub fn new(heaven: u8, hearth: u8, light_source: u8) -> Self {
    Self {
      heaven,
      hearth,
      light_source,
    }
  }
  pub fn dark() -> Self {
    Self {
      heaven: 0,
      hearth: 0,
      light_source: 0,
    }
  }
}

pub fn do_relight(coord: DDD, game_world: &mut GameWorld, remesh: &mut HashSet<DDD>, queue: &mut VecDeque<DDD>) {
  if let Some(light_level) = game_world.get_light_level(coord) && let Some(block) = game_world.get(coord) {
    if block.visible() {
      return;
    }
    let (heavens, hearths): (Vec<_>, Vec<_>) = coord
      .immediate_neighbours()
      .filter_map(|neighbour| game_world.get_light_level(neighbour))
      .map(|light_level| (light_level.heaven, light_level.hearth))
      .unzip();
    let max_heaven = heavens
      .iter()
      .chain([
        {
          let above = game_world
            .get_light_level((coord.0, coord.1+1, coord.2))
            .map(|light_level|light_level.heaven)
            .unwrap_or(0);
          if above == 16 && coord.1 >= 29 {
            &17u8
          } else {
            &0u8
          }
        }
      ])
      .max().unwrap_or(&0).saturating_sub(1);
    let max_hearth = hearths.iter().max().unwrap_or(&0).saturating_sub(1);
    let changed = (light_level.light_source < max_hearth.max(light_level.hearth) && light_level.hearth != max_hearth) || (light_level.heaven != max_heaven);
    if changed {
      game_world.set_light_level(coord, LightLevel::new(max_heaven, light_level.light_source.max(light_level.hearth.max(max_hearth)), light_level.light_source));
      remesh.insert(coord);
      for neighbour in coord.immediate_neighbours() {
        if !game_world.get(neighbour).map(|block|block.visible()).unwrap_or(true) {
          queue.push_front(neighbour);
        }
      }
    }
  }
}

pub fn relight_helper(relight_events: &mut EventReader<RelightEvent>, game_world: &mut GameWorld) -> HashSet<DDD> {
  let mut remesh = HashSet::new();
  for RelightEvent::Relight(coord) in relight_events.iter() {
    remesh.insert(*coord);
    let mut queue = VecDeque::new();
    if game_world.get(*coord).map(|block| block.visible()).unwrap_or(false) {
      coord.immediate_neighbours().for_each(|coord| queue.push_back(coord));
    } else {
      queue.push_back(*coord);
    }
    while let Some(coord) = queue.pop_front() {
      do_relight(coord, game_world, &mut remesh, &mut queue);
    }
  }
  remesh
}
