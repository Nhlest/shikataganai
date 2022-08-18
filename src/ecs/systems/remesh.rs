use crate::ecs::components::blocks::BlockRenderInfo;
use crate::ecs::plugins::rendering::mesh_pipeline::loader::GltfMeshStorageHandle;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::RemeshEvent;
use crate::ecs::resources::chunk_map::BlockAccessor;
use crate::ecs::resources::chunk_map::BlockAccessorStatic;
use crate::util::array::{from_ddd, ArrayIndex};
use crate::GltfMeshStorage;
use bevy::prelude::*;
use itertools::Itertools;
use num_traits::FloatConst;

pub fn remesh_system_auxiliary(
  mut commands: Commands,
  mesh_query: Query<&Handle<Mesh>>,
  mut transform_query: Query<&mut Transform>,
  mut block_accessor: BlockAccessorStatic,
  mut remesh_events: EventReader<RemeshEvent>,
  storage: Res<GltfMeshStorageHandle>,
  mesh_storage_assets: Res<Assets<GltfMeshStorage>>,
) {
  for ch in remesh_events
    .iter()
    .filter_map(|p| if let RemeshEvent::Remesh(d) = p { Some(d) } else { None })
    .unique()
  {
    if !block_accessor.chunk_map.map.contains_key(ch) {
      continue;
    }
    if block_accessor.chunk_map.map[ch].entity.is_none() {
      continue;
    }
    let entity = block_accessor.chunk_map.map[ch].entity.unwrap();
    let bounds = block_accessor.chunks.get(entity).unwrap().grid.bounds;
    let mut i = bounds.0;
    loop {
      let mut block = block_accessor.get_mut(i).unwrap();
      match block.render_info() {
        BlockRenderInfo::AsMesh(mesh) => {
          if block.entity == Entity::from_bits(0) {
            if let Some(mesh_assets_hash_map) = mesh_storage_assets.get(&storage.0) {
              let mesh = &mesh_assets_hash_map[&mesh];
              let render_mesh: &Handle<Mesh> = mesh.render.as_ref().unwrap();
              let rotation = block.meta.get_rotation();
              let e = commands
                .spawn()
                .insert(render_mesh.clone())
                .insert(
                  Transform::from_translation(from_ddd(i) + Vec3::new(0.5, 0.5, 0.5))
                    .with_rotation(Quat::from_rotation_y(f32::PI() / 2.0 * rotation as i32 as f32)),
                )
                .insert(GlobalTransform::default())
                .id();
              block.entity = e;
            }
          } else {
            if mesh_query.get(block.entity).is_ok() {
              transform_query.get_mut(block.entity).unwrap().translation = from_ddd(i) + Vec3::new(0.5, 0.5, 0.5);
            } else {
              if let Some(mesh_assets_hash_map) = mesh_storage_assets.get(&storage.0) {
                let mesh = &mesh_assets_hash_map[&mesh];
                let render_mesh = mesh.render.as_ref().unwrap();
                commands
                  .entity(block.entity)
                  .insert(render_mesh.clone())
                  .insert(Transform::from_translation(from_ddd(i) + Vec3::new(0.5, 0.5, 0.5)))
                  .insert(GlobalTransform::default());
              }
            }
          }
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
