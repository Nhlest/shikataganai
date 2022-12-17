use crate::ecs::plugins::game::{in_game, in_game_extract};
use crate::ecs::plugins::rendering::mesh_pipeline::draw_command::DrawMeshFull;
use crate::ecs::plugins::rendering::mesh_pipeline::loader::{GltfLoaderII, GltfMeshStorage, GltfMeshStorageHandle};
use crate::ecs::plugins::rendering::mesh_pipeline::pipeline::{MeshPipeline, RenderTextures};
use crate::ecs::plugins::rendering::mesh_pipeline::systems::{
  extract_meshes, prepare_textures, queue_light_levels_bind_group, queue_light_texture_bind_group,
  queue_mesh_position_bind_group, queue_meshes, PositionUniform,
};
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::extract_component::UniformComponentPlugin;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_asset::RenderAssetPlugin;
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::render_resource::SpecializedRenderPipelines;
use bevy::render::{RenderApp, RenderStage};
use iyes_loopless::prelude::IntoConditionalSystem;
use shikataganai_common::ecs::resources::light::LightLevel;

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

#[derive(Resource)]
pub struct AmongerTextureHandle(pub Handle<Image>);

impl FromWorld for AmongerTextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    AmongerTextureHandle(asset_server.load("amonger.png"))
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
      // .add_plugin(RenderAssetPlugin::<Image>::default())
      .add_plugin(UniformComponentPlugin::<PositionUniform>::default())
      .add_plugin(UniformComponentPlugin::<LightLevel>::default())
      .add_plugin(RenderAssetPlugin::<GltfMeshStorage>::default())
      .add_plugin(ExtractResourcePlugin::<GltfMeshStorageHandle>::default())
      .add_asset::<GltfMeshStorage>()
      .init_resource::<AmongerTextureHandle>()
      .init_asset_loader::<GltfLoaderII>()
      .init_resource::<GltfMeshStorageHandle>();

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app
      .init_resource::<MeshPipeline>()
      .init_resource::<SpecializedRenderPipelines<MeshPipeline>>()
      .init_resource::<RenderTextures>()
      .add_system_to_stage(RenderStage::Extract, extract_meshes.run_if(in_game_extract))
      .add_system_to_stage(RenderStage::Prepare, prepare_textures)
      .add_system_to_stage(RenderStage::Queue, queue_mesh_position_bind_group.run_if(in_game))
      .add_system_to_stage(RenderStage::Queue, queue_light_levels_bind_group.run_if(in_game))
      .add_system_to_stage(RenderStage::Queue, queue_light_texture_bind_group.run_if(in_game))
      .add_system_to_stage(RenderStage::Queue, queue_meshes.run_if(in_game))
      .add_render_command::<Opaque3d, DrawMeshFull>();
  }
}
