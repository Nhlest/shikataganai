use crate::ecs::components::blocks::{
  animate, AnimationTrait, BlockRenderInfo, BlockTraitExt, ChestAnimations, Skeletons,
};
use crate::ecs::plugins::game::ShikataganaiGameState;
use crate::ecs::plugins::rendering::mesh_pipeline::loader::Meshes;
use bevy::prelude::{Commands, Entity};
use bevy_renet::renet::RenetClient;
use bincode::serialize;
use iyes_loopless::prelude::NextState;
use shikataganai_common::networking::{ClientChannel, PlayerCommand};
use shikataganai_common::util::array::DDD;
use crate::ecs::systems::user_interface::{InventoryItemMovementStatus, InventoryOpened};

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
  fn right_click_interface(
    &self,
    entity: Entity,
    location: DDD,
    commands: &mut Commands,
    client: &mut RenetClient,
  ) -> Option<()> {
    commands.insert_resource(InventoryOpened(entity));
    commands.insert_resource(InventoryItemMovementStatus::Nothing);
    commands.insert_resource(NextState(ShikataganaiGameState::InterfaceOpened));

    animate(commands, entity, ChestAnimations::Open.get_animation());
    client.send_message(
      ClientChannel::ClientCommand.id(),
      serialize(&PlayerCommand::AnimationStart {
        location,
        animation: ChestAnimations::Open.get_animation(),
      })
      .unwrap(),
    );
    Some(())
  }
}
