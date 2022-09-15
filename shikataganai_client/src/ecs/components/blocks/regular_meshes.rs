use crate::ecs::components::blocks::{BlockRenderInfo, BlockTraitExt, Skeletons};
use crate::ecs::plugins::rendering::mesh_pipeline::loader::Meshes;
use crate::ecs::systems::user_interface::chest_inventory::InventoryOpened;
use bevy::prelude::{Commands, Entity};

pub struct Stair;
pub struct Chest;

impl BlockTraitExt for Stair {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::AsMesh(Meshes::Stair)
  }
}

impl BlockTraitExt for Chest {
  fn render_info(&self) -> BlockRenderInfo {
    BlockRenderInfo::AsSkeleton(Skeletons::Chest)
  }
  fn right_click_interface(&self, entity: Entity, commands: &mut Commands) -> Option<()> {
    commands.insert_resource(InventoryOpened(entity));
    Some(())
  }
}
