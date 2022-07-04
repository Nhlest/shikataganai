use bevy::prelude::*;
use crate::ecs::components::chunk::Chunk;
use crate::ecs::plugins::voxel::{RelightType, RemeshEvent};
use crate::ecs::resources::chunk_map::{ChunkMap, LightPropagationType};
use crate::ecs::resources::light::LightLevel;
use crate::util::array::ImmediateNeighbours;

pub fn relight_system(
  mut commands: Commands,
  mut chunk_map: ResMut<ChunkMap>,
  mut chunks: Query<&mut Chunk>,
  mut events: EventReader<RemeshEvent>,
) {
  for event in events.iter() {
    if let RemeshEvent::Relight(r_type, ddd) = event {
      match r_type {
        RelightType::LightSourceAdded => {}
        RelightType::LightSourceRemoved => {}
        RelightType::BlockAdded => {
          // let light = chunk_map.replace_light_level(&mut chunks, *ddd, LightLevel::dark());
          // for i in ddd.immeidate_neighbours() {
          //   chunk_map.propagate_light(&mut chunks, i, LightPropagationType::Brighten)
          // }
          // ddd.immediate_neighbours()
        }
        RelightType::BlockRemoved => {}
      }
    }
  }
}