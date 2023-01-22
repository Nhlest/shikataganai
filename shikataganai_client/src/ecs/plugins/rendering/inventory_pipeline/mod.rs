use crate::ecs::plugins::game::{in_game, in_game_extract};
// use crate::ecs::plugins::imgui::{IMGUI_PASS, TEXTURE_NODE_INPUT_SLOT};
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::ExtractedItems;
use crate::ecs::plugins::rendering::inventory_pipeline::pipeline::InventoryNode;
use crate::ecs::plugins::rendering::inventory_pipeline::systems::{
  clear_renderapp_extraction, clear_rerender, extract_inventory_tiles, update_extracted_items,
};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_resource::{Extent3d, PipelineCache, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::{RenderApp, RenderStage};
use bevy_egui::EguiContext;
use egui::TextureId;
use iyes_loopless::prelude::IntoConditionalSystem;
use std::ops::{Deref, DerefMut};
use wgpu::TextureDescriptor;

pub mod inventory_cache;
pub mod node;
pub mod pipeline;
pub mod systems;

pub struct InventoryRendererPlugin;

#[derive(Resource, ExtractResource, Clone)]
pub struct InventoryTextureOutputHandle(pub Handle<Image>, pub TextureId);

pub const INVENTORY_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151595899);
pub const INVENTORY_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151596999);
pub const INVENTORY_MESH_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151295899);
pub const INVENTORY_MESH_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151196999);

pub const INVENTORY_PASS: &str = "Inventory Pass";
pub const TEXTURE_NODE_OUTPUT_SLOT: &str = "Texture Slot Output";

pub const INVENTORY_OUTPUT_TEXTURE_WIDTH: f32 = 8.0;

impl Plugin for InventoryRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let voxel_shader_vertex =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/offscreen.vert.spv").as_slice());
    let voxel_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/offscreen.frag.spv").as_slice());
    shaders.set_untracked(INVENTORY_SHADER_VERTEX_HANDLE, voxel_shader_vertex);
    shaders.set_untracked(INVENTORY_SHADER_FRAGMENT_HANDLE, voxel_shader_fragment);
    let voxel_shader_vertex =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/inventory_mesh.vert.spv").as_slice());
    let voxel_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/inventory_mesh.frag.spv").as_slice());
    shaders.set_untracked(INVENTORY_MESH_SHADER_VERTEX_HANDLE, voxel_shader_vertex);
    shaders.set_untracked(INVENTORY_MESH_SHADER_FRAGMENT_HANDLE, voxel_shader_fragment);

    app.init_resource::<ExtractedItems>();
    app.add_system_to_stage(CoreStage::First, clear_rerender);
    app.add_system_to_stage(CoreStage::Last, update_extracted_items);

    let mut images = app.world.resource_mut::<Assets<Image>>();
    let mut image = Image {
      texture_descriptor: TextureDescriptor {
        label: "offscreen_texture".into(),
        size: Extent3d {
          width: 1024,
          height: 1024,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::bevy_default(),
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
      },
      ..default()
    };

    image.resize(Extent3d {
      width: 1024,
      height: 1024,
      depth_or_array_layers: 1,
    });
    let handle = images.add(image).clone();

    let mut egui = app.world.resource_mut::<EguiContext>();
    let i = egui.add_image(handle.clone());
    app.world.insert_resource(InventoryTextureOutputHandle(handle, i));
    app.add_plugin(ExtractResourcePlugin::<InventoryTextureOutputHandle>::default());

    if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
      let inventory_node = {
        let world_cell = render_app.world.cell();
        let mut render_pipeline_cache = world_cell.resource_mut::<PipelineCache>();
        let render_device = world_cell.resource::<RenderDevice>();
        InventoryNode::new(render_device.deref(), render_pipeline_cache.deref_mut())
      };

      render_app.add_system_to_stage(RenderStage::Extract, extract_inventory_tiles.run_if(in_game_extract));
      render_app.add_system_to_stage(RenderStage::Cleanup, clear_renderapp_extraction.run_if(in_game));

      let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
      render_graph.add_node(INVENTORY_PASS, inventory_node);
    }
  }
}
