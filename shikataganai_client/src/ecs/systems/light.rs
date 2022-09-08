use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::LightTextureHandle;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::{RemeshEvent};
use bevy::prelude::*;
use bevy::utils::HashSet;
use itertools::Itertools;
use shikataganai_common::ecs::resources::light::{relight_helper, RelightEvent};
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::util::array::{FlatFullNeighbours, ImmediateNeighbours};

pub fn religh_system(
  mut relight: EventReader<RelightEvent>,
  mut remesh: EventWriter<RemeshEvent>,
  mut game_world: ResMut<GameWorld>
) {
  for coord in relight_helper(&mut relight, game_world.as_mut())
    .iter() {
    coord.flat_full_neighbours().map(|coord|GameWorld::get_chunk_coord(coord)).unique().for_each(|chunk_coord| remesh.send(RemeshEvent::Remesh(chunk_coord)));
  }
}
//
// #[inline]
// fn compute_light(
//   heaven: u8,
//   heaven_intencity: f32,
//   heaven_intencity_time: f32,
//   hearth: u8,
//   hearth_intencity: f32,
// ) -> u8 {
//   let mix = 0.25;
//   let h1 = (heaven as f32 * heaven_intencity_time * heaven_intencity) * 1.0;
//   let h2 = (hearth as f32 * hearth_intencity) * 1.0;
//   return (h1 * 0.5 / mix + h2 * 0.5 * (1.0 - 0.5) / mix).floor() as u8;
// }

// // TODO: implement
// #[allow(dead_code)]
// pub fn recalculate_light_map(
//   light_texture_handle: Res<LightTextureHandle>,
//   mut images: ResMut<Assets<Image>>,
//   mut time: Local<f32>,
//   mut clear_color: ResMut<ClearColor>,
// ) {
//   *time = *time + 0.01;
//   let heaven_light_time = (time.sin() + 1.0) / 2.0;
//   clear_color.0 = Color::Rgba {
//     red: 0.527 * (heaven_light_time * 0.75 + 0.25),
//     green: 0.804 * (heaven_light_time * 0.75 + 0.25),
//     blue: 0.917 * (heaven_light_time * 0.75 + 0.25),
//     alpha: 1.0,
//   };
//   let heaven_light = [255, 255, 255];
//   let hearth_light = [255, 214, 214];
//   if let Some(texture_image) = images.get_mut(&light_texture_handle.0) {
//     for heaven in 0..16 {
//       for hearth in 0..16 {
//         let c = (heaven * 16 + hearth) * 4;
//         let heaven_intensity = 0.8f32.powi(15 - heaven);
//         let hearth_intensity = 0.8f32.powi(15 - hearth);
//         texture_image.data[c as usize + 0] = compute_light(
//           heaven_light[0],
//           heaven_intensity,
//           heaven_light_time,
//           hearth_light[0],
//           hearth_intensity,
//         );
//         texture_image.data[c as usize + 1] = compute_light(
//           heaven_light[1],
//           heaven_intensity,
//           heaven_light_time,
//           hearth_light[1],
//           hearth_intensity,
//         );
//         texture_image.data[c as usize + 2] = compute_light(
//           heaven_light[2],
//           heaven_intensity,
//           heaven_light_time,
//           hearth_light[2],
//           hearth_intensity,
//         );
//         texture_image.data[c as usize + 3] = 255;
//       }
//     }
//   }
// }
