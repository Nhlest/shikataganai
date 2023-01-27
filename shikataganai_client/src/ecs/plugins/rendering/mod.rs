use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use crate::ecs::plugins::rendering::inventory_pipeline::InventoryRendererPlugin;
use crate::ecs::plugins::rendering::mesh_pipeline::MeshRendererPlugin;
use crate::ecs::plugins::rendering::particle_pipeline::ParticleRendererPlugin;
use crate::ecs::plugins::rendering::skybox_pipeline::SkyboxRendererPlugin;
use crate::ecs::plugins::rendering::voxel_pipeline::VoxelRendererPlugin;

pub mod draw_command;

pub mod inventory_pipeline;
pub mod mesh_pipeline;
pub mod skybox_pipeline;
pub mod voxel_pipeline;
pub mod particle_pipeline;

pub struct ShikataganaiRendererPlugins;

impl PluginGroup for ShikataganaiRendererPlugins {
  fn build(self) -> PluginGroupBuilder {
    PluginGroupBuilder::start::<Self>()
      .add(VoxelRendererPlugin)
      .add(MeshRendererPlugin)
      .add(InventoryRendererPlugin)
      .add(SkyboxRendererPlugin)
      .add(ParticleRendererPlugin)
  }
}
