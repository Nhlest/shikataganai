use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use crate::ecs::plugins::rendering::inventory_pipeline::InventoryRendererPlugin;
use crate::ecs::plugins::rendering::mesh_pipeline::MeshRendererPlugin;
use crate::ecs::plugins::rendering::voxel_pipeline::VoxelRendererPlugin;

pub mod draw_command;

pub mod inventory_pipeline;
pub mod mesh_pipeline;
pub mod voxel_pipeline;

pub struct ShikataganaiRendererPlugins;

impl PluginGroup for ShikataganaiRendererPlugins {
  fn build(&mut self, group: &mut PluginGroupBuilder) {
    group
      .add(VoxelRendererPlugin)
      .add(MeshRendererPlugin)
      .add(InventoryRendererPlugin);
  }
}
