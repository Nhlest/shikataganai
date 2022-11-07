use crate::ecs::components::blocks::DerefExt;
use crate::ecs::components::blocks::{BlockRenderInfo, Skeleton};
use crate::ecs::plugins::rendering::mesh_pipeline::loader::GltfMeshStorageHandle;
use crate::ecs::plugins::rendering::mesh_pipeline::systems::MeshMarker;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::RemeshEvent;
use crate::GltfMeshStorage;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use itertools::Itertools;
use num_traits::FloatConst;
use shikataganai_common::ecs::components::blocks::ReverseLocation;
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::util::array::{from_ddd, ArrayIndex};

pub fn remesh_system_auxiliary(
  mut commands: Commands,
  mesh_query: Query<&Handle<Mesh>>,
  skeleton_query: Query<&Skeleton>,
  mut transform_query: Query<&mut Transform>,
  mut game_world: ResMut<GameWorld>,
  mut remesh_events: EventReader<RemeshEvent>,
  storage: Res<GltfMeshStorageHandle>,
  mesh_storage_assets: Res<Assets<GltfMeshStorage>>,
) {
  for ch in remesh_events
    .iter()
    .filter_map(|p| if let RemeshEvent::Remesh(d) = p { Some(d) } else { None })
    .unique()
  {
    if !game_world.chunks.contains_key(ch) {
      continue;
    }
    let bounds = game_world.chunks.get(ch).unwrap().grid.bounds;
    let mut i = bounds.0;
    loop {
      let mut block = game_world.get_mut(i).unwrap();
      if block.need_reverse_location() {
        block.entity = if block.entity == Entity::from_bits(0) {
          commands.spawn()
        } else {
          commands.entity(block.entity)
        }
        .insert(ReverseLocation(i))
        .id();
      }
      match block.deref_ext().render_info() {
        BlockRenderInfo::AsMesh(mesh) => {
          if block.entity == Entity::from_bits(0) {
            if let Some(mesh_assets_hash_map) = mesh_storage_assets.get(&storage.0) {
              let mesh = &mesh_assets_hash_map[&mesh];
              let render_mesh: &Handle<Mesh> = mesh.render.as_ref().unwrap();
              let rotation = block.meta.get_rotation();
              let e = commands
                .spawn()
                .insert(render_mesh.clone())
                .insert(MeshMarker)
                .insert(
                  Transform::from_translation(from_ddd(i) + Vec3::new(0.5, 0.5, 0.5))
                    .with_rotation(Quat::from_rotation_y(f32::PI() / 2.0 * rotation as i32 as f32)),
                )
                .insert(GlobalTransform::default())
                .id();
              block.entity = e;
            }
          } else if mesh_query.get(block.entity).is_ok() {
            transform_query.get_mut(block.entity).unwrap().translation = from_ddd(i) + Vec3::new(0.5, 0.5, 0.5);
          } else {
            if let Some(mesh_assets_hash_map) = mesh_storage_assets.get(&storage.0) {
              let mesh = &mesh_assets_hash_map[&mesh];
              let render_mesh = mesh.render.as_ref().unwrap();
              commands
                .entity(block.entity)
                .insert(MeshMarker)
                .insert(render_mesh.clone())
                .insert(Transform::from_translation(from_ddd(i) + Vec3::new(0.5, 0.5, 0.5)))
                .insert(GlobalTransform::default());
            }
          }
        }
        BlockRenderInfo::AsSkeleton(skeleton) => {
          let rotation = block.meta.get_rotation();
          if block.entity == Entity::from_bits(0) {
            Some(commands.spawn())
          } else if skeleton_query.get(block.entity).is_ok() {
            transform_query.get_mut(block.entity).unwrap().translation = from_ddd(i) + Vec3::new(0.5, 0.5, 0.5);
            None
          } else {
            Some(commands.entity(block.entity))
          }
          .map(|mut commands| {
            if let Some(mesh_assets_hash_map) = mesh_storage_assets.get(&storage.0) {
              let mut hash_map = HashMap::new();
              let id = commands
                .insert(
                  Transform::from_translation(from_ddd(i) + Vec3::new(0.5, 0.5, 0.5))
                    .with_rotation(Quat::from_rotation_y(f32::PI() / 2.0 * rotation as i32 as f32)),
                )
                .insert(GlobalTransform::default())
                .with_children(|c| {
                  for (i, mesh_def) in skeleton.to_skeleton_def().skeleton {
                    let mesh = &mesh_assets_hash_map[&mesh_def.mesh];
                    let render_mesh = mesh.render.as_ref().unwrap();
                    let transform = Transform::from_translation(mesh_def.offset);
                    let id = c
                      .spawn()
                      .insert(MeshMarker)
                      .insert(render_mesh.clone())
                      .insert(transform)
                      .insert(GlobalTransform::default())
                      .id();
                    hash_map.insert(i, id);
                  }
                })
                .insert(Skeleton { skeleton: hash_map })
                .id();
              block.entity = id;
            }
          });
        }
        _ => {}
      }
      i = match i.next(&bounds) {
        None => break,
        Some(i) => i,
      }
    }
  }
}
