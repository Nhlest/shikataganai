
use crate::ecs::plugins::game::{in_game, in_game_extract};
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{ArrayTextureHandle, ItemTextureHandle, LightTextureHandle, ParticleTextureHandle, TextureHandle};
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
use crate::ecs::plugins::rendering::particle_pipeline::systems::{extract_aspect_ratio, extract_particles, particle_system, queue_particles};
use bytemuck::{Pod, Zeroable};

pub mod bind_groups;
pub mod draw_command;
pub mod pipeline;
pub mod systems;

pub const PARTICLE_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151697799);
pub const PARTICLE_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151697999);

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum EffectSprite {
  Smoke
}

#[derive(Component, Clone)]
pub struct Particle {
  pub location: Vec3,
  pub tile: EffectSprite,
  pub lifetime: u64,
  pub velocity: Vec3
}

#[derive(Component, Clone)]
pub struct ParticleEmitter {
  pub location: Vec3,
  pub tile: EffectSprite,
  pub lifetime: u64,
}

#[derive(Pod, Zeroable, Copy, Clone, Component)]
#[repr(C)]
pub struct ParticleVertex {
  pub location: Vec3,
  pub tile: u32,
  pub heaven: u16,
  pub hearth: u16,
}

#[derive(Resource)]
pub struct ParticleBuffer {
  pub particles: BufferVec<ParticleVertex>,
  pub count: usize
}

pub struct ParticleRendererPlugin;

impl Plugin for ParticleRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let particle_shader_vertex =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/particle.vert.spv").as_slice());
    let particle_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/particle.frag.spv").as_slice());
    shaders.set_untracked(PARTICLE_SHADER_VERTEX_HANDLE, particle_shader_vertex);
    shaders.set_untracked(PARTICLE_SHADER_FRAGMENT_HANDLE, particle_shader_fragment);

    let on_game_simulation_continuous = ConditionSet::new()
      .run_if(in_game)
      .with_system(particle_system)
      .into();

    let on_game_simulation_extract = ConditionSet::new()
      .run_if(in_game_extract)
      .with_system(extract_particles)
      .into();

    app
      .init_resource::<ParticleTextureHandle>()
      .add_system_set(on_game_simulation_continuous);

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();

    let mut buf = ParticleBuffer {
      particles: BufferVec::new(BufferUsages::VERTEX),
      count: 0,
    };
    let render_device = render_app.world.resource::<RenderDevice>();
    buf.particles.reserve(1, render_device);
    render_app.insert_resource(buf);

    render_app
      .init_resource::<ParticlePipeline>()
      .init_resource::<SpecializedRenderPipelines<ParticlePipeline>>()
      .add_system_set_to_stage(RenderStage::Extract, on_game_simulation_extract)
      .add_system_to_stage(RenderStage::Extract, extract_aspect_ratio)
      .add_system_to_stage(RenderStage::Queue, queue_particles)
      .add_render_command::<Opaque3d, DrawParticlesFull>();
  }
}