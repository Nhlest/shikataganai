use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::LightTextureHandle;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::{RelightEvent, RelightType, RemeshEvent};
use crate::ecs::resources::chunk_map::BlockAccessor;
use crate::ecs::resources::chunk_map::{BlockAccessorStatic, ChunkMap};
use crate::ecs::resources::light::LightLevel;
use bevy::prelude::*;
use bevy::utils::HashSet;
use shikataganai_common::util::array::ImmediateNeighbours;

pub fn relight_system(
  mut remesh_events: EventWriter<RemeshEvent>,
  mut relight_events: EventReader<RelightEvent>,
  mut block_accessor: BlockAccessorStatic,
) {
  let mut remesh = HashSet::new();
  for event in relight_events.iter() {
    if let RelightEvent::Relight(r_type, ddd) = event {
      match r_type {
        RelightType::LightSourceAdded => {
          let l = block_accessor.get_light_level(*ddd).unwrap();
          block_accessor.set_light_level(*ddd, LightLevel::new(l.heaven, 15));
          remesh.insert(ChunkMap::get_chunk_coord(*ddd));
          block_accessor.propagate_light(*ddd, &mut remesh);
          for i in ddd.immediate_neighbours() {
            block_accessor.propagate_light(i, &mut remesh);
          }
        }
        // RelightType::LightSourceRemoved => {}
        RelightType::BlockAdded => {
          block_accessor.set_light_level(*ddd, LightLevel::dark());
          remesh.insert(ChunkMap::get_chunk_coord(*ddd));
          block_accessor.propagate_light(*ddd, &mut remesh);
          for i in ddd.immediate_neighbours() {
            block_accessor.propagate_light(i, &mut remesh);
          }
        }
        RelightType::BlockRemoved => {
          remesh.insert(ChunkMap::get_chunk_coord(*ddd));
          block_accessor.propagate_light(*ddd, &mut remesh);
          for i in ddd.immediate_neighbours() {
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

#[inline]
fn compute_light(
  heaven: u8,
  heaven_intencity: f32,
  heaven_intencity_time: f32,
  hearth: u8,
  hearth_intencity: f32,
) -> u8 {
  let mix = 0.25;
  let h1 = (heaven as f32 * heaven_intencity_time * heaven_intencity) * 1.0;
  let h2 = (hearth as f32 * hearth_intencity) * 1.0;
  return (h1 * 0.5 / mix + h2 * 0.5 * (1.0 - 0.5) / mix).floor() as u8;
}

// TODO: implement
#[allow(dead_code)]
pub fn recalculate_light_map(
  light_texture_handle: Res<LightTextureHandle>,
  mut images: ResMut<Assets<Image>>,
  mut time: Local<f32>,
  mut clear_color: ResMut<ClearColor>,
) {
  *time = *time + 0.01;
  let heaven_light_time = (time.sin() + 1.0) / 2.0;
  clear_color.0 = Color::Rgba {
    red: 0.527 * (heaven_light_time * 0.75 + 0.25),
    green: 0.804 * (heaven_light_time * 0.75 + 0.25),
    blue: 0.917 * (heaven_light_time * 0.75 + 0.25),
    alpha: 1.0,
  };
  let heaven_light = [255, 255, 255];
  let hearth_light = [255, 214, 214];
  if let Some(texture_image) = images.get_mut(&light_texture_handle.0) {
    for heaven in 0..16 {
      for hearth in 0..16 {
        let c = (heaven * 16 + hearth) * 4;
        let heaven_intensity = 0.8f32.powi(15 - heaven);
        let hearth_intensity = 0.8f32.powi(15 - hearth);
        texture_image.data[c as usize + 0] = compute_light(
          heaven_light[0],
          heaven_intensity,
          heaven_light_time,
          hearth_light[0],
          hearth_intensity,
        );
        texture_image.data[c as usize + 1] = compute_light(
          heaven_light[1],
          heaven_intensity,
          heaven_light_time,
          hearth_light[1],
          hearth_intensity,
        );
        texture_image.data[c as usize + 2] = compute_light(
          heaven_light[2],
          heaven_intensity,
          heaven_light_time,
          hearth_light[2],
          hearth_intensity,
        );
        texture_image.data[c as usize + 3] = 255;
      }
    }
  }
}
