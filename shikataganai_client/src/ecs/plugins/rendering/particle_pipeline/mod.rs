
use crate::ecs::plugins::game::{in_game, in_game_extract};
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{
  ArrayTextureHandle, ItemTextureHandle, LightTextureHandle, TextureHandle,
};
use crate::ecs::plugins::rendering::voxel_pipeline::draw_command::DrawVoxelsFull;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::RemeshEvent;
use crate::ecs::plugins::rendering::voxel_pipeline::pipeline::VoxelPipeline;
use crate::ecs::plugins::rendering::voxel_pipeline::systems::{
  extract_chunks, queue_chunks, ExtractedBlocks, OverlayBuffer,
};
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::render_resource::{BufferVec, SpecializedRenderPipelines};
use bevy::render::renderer::RenderDevice;
use bevy::render::{Extract, RenderApp, RenderStage};
use iyes_loopless::prelude::{ConditionSet, IntoConditionalSystem};
use shikataganai_common::ecs::resources::light::RelightEvent;
use wgpu::{BufferUsages, Extent3d};
use shikataganai_common::util::array::DDD;
use crate::ecs::plugins::rendering::particle_pipeline::draw_command::DrawParticlesFull;
use crate::ecs::plugins::rendering::particle_pipeline::pipeline::ParticlePipeline;

pub mod bind_groups;
pub mod draw_command;
pub mod pipeline;
pub mod systems;

pub const PARTICLE_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151697799);
pub const PARTICLE_SHADER_GEOMETRY_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151697899);
pub const PARTICLE_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151697999);

pub enum EffectSprite {

}

pub struct Particle {
  pub tile: EffectSprite,
  pub location: DDD,
}

pub struct ParticleRendererPlugin;

impl Plugin for ParticleRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let particle_shader_vertex =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/particle.vert.spv").as_slice());
    let particle_shader_geometry =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/particle.geom.spv").as_slice());
    let particle_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/particle.frag.spv").as_slice());
    shaders.set_untracked(PARTICLE_SHADER_VERTEX_HANDLE, particle_shader_vertex);
    shaders.set_untracked(PARTICLE_SHADER_GEOMETRY_HANDLE, particle_shader_geometry);
    shaders.set_untracked(PARTICLE_SHADER_FRAGMENT_HANDLE, particle_shader_fragment);

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();

    render_app
      .init_resource::<ParticlePipeline>()
      .init_resource::<SpecializedRenderPipelines<ParticlePipeline>>()
      .add_render_command::<Opaque3d, DrawParticlesFull>();
  }
}