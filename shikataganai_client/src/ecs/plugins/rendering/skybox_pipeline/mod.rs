use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::render_resource::SpecializedRenderPipelines;
use bevy::render::{RenderApp, RenderStage};
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::render_phase::AddRenderCommand;
use bevy_atmosphere::skybox::AtmosphereSkyBoxMaterial;
use iyes_loopless::prelude::IntoConditionalSystem;
use crate::ecs::plugins::game::{in_game, in_game_extract};
use crate::ecs::plugins::rendering::skybox_pipeline::draw_command::DrawSkyboxFull;
use crate::ecs::plugins::rendering::skybox_pipeline::pipeline::SkyboxPipeline;
use crate::ecs::plugins::rendering::skybox_pipeline::systems::{extract_skybox_material_handle, queue_skybox, queue_skybox_mesh_position_bind_group};

pub mod draw_command;
pub mod pipeline;
pub mod systems;
pub mod bind_groups;

pub struct SkyboxRendererPlugin;

pub const INVENTORY_PASS: &'static str = "Inventory Pass";

impl Plugin for SkyboxRendererPlugin {
  fn build(&self, app: &mut App) {
    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app
      .init_resource::<SkyboxPipeline>()
      .init_resource::<SpecializedRenderPipelines<SkyboxPipeline>>()
      .add_system_to_stage(RenderStage::Extract, extract_skybox_material_handle.run_if(in_game_extract))
      .add_system_to_stage(RenderStage::Queue, queue_skybox.run_if(in_game))
      .add_system_to_stage(RenderStage::Queue, queue_skybox_mesh_position_bind_group.run_if(in_game))
      .add_render_command::<Opaque3d, DrawSkyboxFull>();
  }
}
