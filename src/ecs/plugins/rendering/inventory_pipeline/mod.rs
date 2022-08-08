use crate::ecs::components::block_or_item::BlockOrItem;
use crate::ecs::plugins::game::{in_game, in_game_extract};
use crate::ecs::plugins::imgui::{IMGUI_PASS, TEXTURE_NODE_INPUT_SLOT};
use crate::ecs::plugins::rendering::inventory_pipeline::pipeline::InventoryNode;
use crate::ecs::plugins::rendering::inventory_pipeline::systems::{
  cleanup_rerender, extract_inventory_tiles, prepare_extracted_inventory,
};
use crate::ecs::resources::player::RerenderInventory;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_resource::PipelineCache;
use bevy::render::renderer::RenderDevice;
use bevy::render::{RenderApp, RenderStage};
use bevy::utils::hashbrown::HashMap;
use iyes_loopless::prelude::IntoConditionalSystem;
use std::ops::{Deref, DerefMut};

pub mod node;
pub mod pipeline;
pub mod systems;

pub struct InventoryRendererPlugin;

pub const INVENTORY_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151595899);
pub const INVENTORY_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151596999);

pub const INVENTORY_PASS: &'static str = "Inventory Pass";
pub const TEXTURE_NODE_OUTPUT_SLOT: &'static str = "Texture Slot Output";

pub const INVENTORY_OUTPUT_TEXTURE_WIDTH: f32 = 8.0;

#[derive(Default)]
pub struct ExtractedItems(pub HashMap<BlockOrItem, (f32, f32)>);

impl Plugin for InventoryRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let voxel_shader_vertex =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/offscreen.vert.spv").as_slice());
    let voxel_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/offscreen.frag.spv").as_slice());
    shaders.set_untracked(INVENTORY_SHADER_VERTEX_HANDLE, voxel_shader_vertex);
    shaders.set_untracked(INVENTORY_SHADER_FRAGMENT_HANDLE, voxel_shader_fragment);

    app.init_resource::<RerenderInventory>();
    app.init_resource::<ExtractedItems>();
    app.add_system_to_stage(CoreStage::First, cleanup_rerender);
    app.add_system_to_stage(CoreStage::PostUpdate, prepare_extracted_inventory.run_if(in_game));

    if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
      let inventory_node = {
        let world_cell = render_app.world.cell();
        let mut render_pipeline_cache = world_cell.resource_mut::<PipelineCache>();
        let render_device = world_cell.resource::<RenderDevice>();
        InventoryNode::new(render_device.deref(), render_pipeline_cache.deref_mut())
      };

      render_app.add_system_to_stage(RenderStage::Extract, extract_inventory_tiles.run_if(in_game_extract));
      render_app.init_resource::<ExtractedItems>();

      let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
      render_graph.add_node(INVENTORY_PASS, inventory_node);

      render_graph.add_node_edge(INVENTORY_PASS, IMGUI_PASS).unwrap();
      render_graph
        .add_slot_edge(
          INVENTORY_PASS,
          TEXTURE_NODE_OUTPUT_SLOT,
          IMGUI_PASS,
          TEXTURE_NODE_INPUT_SLOT,
        )
        .unwrap();
    }
  }
}
