use crate::ecs::plugins::rendering::mesh_pipeline::draw_command::DrawMeshFull;
use crate::ecs::plugins::rendering::mesh_pipeline::loader::{GltfLoaderII, GltfMeshStorage};
use crate::ecs::plugins::rendering::mesh_pipeline::pipeline::MeshPipeline;
use crate::ecs::plugins::rendering::mesh_pipeline::systems::{
  extract_meshes, queue_mesh_position_bind_group, queue_meshes, PositionUniform,
};
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::extract_component::UniformComponentPlugin;
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::render_resource::SpecializedRenderPipelines;
use bevy::render::{RenderApp, RenderStage};
use bevy_rapier3d::prelude::*;

pub mod bind_groups;
pub mod draw_command;
pub mod loader;
pub mod pipeline;
pub mod systems;

pub struct MeshRendererPlugin;

pub const MESH_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597699);
pub const MESH_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597799);

fn spawn_mesh(
  storage: Res<GltfMeshStorage>,
  mut local: Local<bool>,
  mesh_assets: Res<Assets<Mesh>>,
  mut commands: Commands,
) {
  if !*local {
    if let Some(m) = mesh_assets.get(storage.0.as_ref().unwrap()) {
      commands
        .spawn()
        .insert(storage.0.as_ref().unwrap().clone())
        .insert(Transform::from_xyz(8.0, 39.0, 12.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::from_bevy_mesh(m, &ComputedColliderShape::TriMesh).unwrap())
        .insert(Friction {
          coefficient: 0.0,
          combine_rule: CoefficientCombineRule::Min,
        })
        .insert(SolverGroups::new(0b10, 0b01))
        .insert(CollisionGroups::new(0b10, 0b01))
        .insert(GlobalTransform::default());
      commands
        .spawn()
        .insert(storage.0.as_ref().unwrap().clone())
        .insert(Transform::from_xyz(7.0, 38.0, 12.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::from_bevy_mesh(m, &ComputedColliderShape::TriMesh).unwrap())
        .insert(Friction {
          coefficient: 0.0,
          combine_rule: CoefficientCombineRule::Min,
        })
        .insert(SolverGroups::new(0b10, 0b01))
        .insert(CollisionGroups::new(0b10, 0b01))
        .insert(GlobalTransform::default());
      commands
        .spawn()
        .insert(storage.0.as_ref().unwrap().clone())
        .insert(Transform::from_xyz(6.0, 37.0, 12.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::from_bevy_mesh(m, &ComputedColliderShape::TriMesh).unwrap())
        .insert(Friction {
          coefficient: 0.0,
          combine_rule: CoefficientCombineRule::Min,
        })
        .insert(SolverGroups::new(0b10, 0b01))
        .insert(CollisionGroups::new(0b10, 0b01))
        .insert(GlobalTransform::default());
      commands
        .spawn()
        .insert(storage.0.as_ref().unwrap().clone())
        .insert(Transform::from_xyz(5.0, 36.0, 12.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::from_bevy_mesh(m, &ComputedColliderShape::TriMesh).unwrap())
        .insert(Friction {
          coefficient: 0.0,
          combine_rule: CoefficientCombineRule::Min,
        })
        .insert(SolverGroups::new(0b10, 0b01))
        .insert(CollisionGroups::new(0b10, 0b01))
        .insert(GlobalTransform::default());
      *local = true;
    }
  }
}

impl Plugin for MeshRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let mesh_shader_vertex =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/mesh.vert.spv").as_slice());
    let mesh_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/mesh.frag.spv").as_slice());
    shaders.set_untracked(MESH_SHADER_VERTEX_HANDLE, mesh_shader_vertex);
    shaders.set_untracked(MESH_SHADER_FRAGMENT_HANDLE, mesh_shader_fragment);

    app
      .add_plugin(UniformComponentPlugin::<PositionUniform>::default())
      .init_asset_loader::<GltfLoaderII>()
      .init_resource::<GltfMeshStorage>()
      .add_system(spawn_mesh);

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app
      .init_resource::<MeshPipeline>()
      .init_resource::<SpecializedRenderPipelines<MeshPipeline>>()
      .add_system_to_stage(RenderStage::Extract, extract_meshes)
      .add_system_to_stage(RenderStage::Queue, queue_mesh_position_bind_group)
      .add_system_to_stage(RenderStage::Queue, queue_meshes)
      .add_render_command::<Opaque3d, DrawMeshFull>();
  }
}
