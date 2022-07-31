use crate::ecs::plugins::voxel::{RelightEvent, RelightType, RemeshEvent};
use crate::ecs::resources::chunk_map::BlockAccessor;
use crate::ecs::resources::chunk_map::{BlockAccessorStatic, ChunkMap};
use crate::ecs::resources::light::LightLevel;
use crate::util::array::ImmediateNeighbours;
use bevy::prelude::*;
use bevy::utils::HashSet;

pub fn relight_system(
  mut remesh_events: EventWriter<RemeshEvent>,
  mut relight_events: EventReader<RelightEvent>,
  mut block_accessor: BlockAccessorStatic,
) {
  let mut remesh = HashSet::new();
  for event in relight_events.iter() {
    if let RelightEvent::Relight(r_type, ddd) = event {
      match r_type {
        // RelightType::LightSourceAdded => {
        //   let l = block_accessor.get_light_level(*ddd).unwrap();
        //   block_accessor.set_light_level(*ddd, LightLevel::new(l.heaven, 15));
        //   remesh.insert(ChunkMap::get_chunk_coord(*ddd));
        //   block_accessor.propagate_light(*ddd, &mut remesh);
        //   for i in ddd.immeidate_neighbours() {
        //     block_accessor.propagate_light(i, &mut remesh);
        //   }
        // }
        // RelightType::LightSourceRemoved => {}
        RelightType::BlockAdded => {
          block_accessor.set_light_level(*ddd, LightLevel::dark());
          remesh.insert(ChunkMap::get_chunk_coord(*ddd));
          block_accessor.propagate_light(*ddd, &mut remesh);
          for i in ddd.immeidate_neighbours() {
            block_accessor.propagate_light(i, &mut remesh);
          }
        }
        RelightType::BlockRemoved => {
          remesh.insert(ChunkMap::get_chunk_coord(*ddd));
          block_accessor.propagate_light(*ddd, &mut remesh);
          for i in ddd.immeidate_neighbours() {
            block_accessor.propagate_light(i, &mut remesh);
          }
        }
      }
    }
  }
  remesh
    .iter()
    .for_each(|dd| remesh_events.send(RemeshEvent::Remesh(*dd)));
}
