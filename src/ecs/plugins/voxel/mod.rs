pub mod inventory_pipeline;
pub mod mesh_pipeline;
pub mod misc;
pub mod systems;
pub mod voxel_pipeline;

use crate::ecs::plugins::voxel::inventory_pipeline::OffscreenInventoryAuxRendererPlugin;
pub use crate::ecs::plugins::voxel::mesh_pipeline::*;
pub use crate::ecs::plugins::voxel::misc::*;
pub use crate::ecs::plugins::voxel::systems::*;
pub use crate::ecs::plugins::voxel::voxel_pipeline::*;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::PluginGroup;

pub struct ShikataganaiRendererPlugins;

impl PluginGroup for ShikataganaiRendererPlugins {
  fn build(&mut self, group: &mut PluginGroupBuilder) {
    group
      .add(VoxelRendererPlugin)
      .add(MeshRendererPlugin)
      .add(OffscreenInventoryAuxRendererPlugin);
  }
}
