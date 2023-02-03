use crate::ecs::plugins::game::{in_game, in_game_extract};
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{
  ArrayTextureHandle, ItemTextureHandle, LightTextureHandle, ParticleTextureHandle, TextureHandle,
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

pub mod bind_groups;
pub mod consts;
pub mod draw_command;
pub mod meshing;
pub mod pipeline;
pub mod systems;

pub const VOXEL_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597899);
pub const VOXEL_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597999);

pub struct VoxelRendererPlugin;

fn create_texture_array_system(
  mut commands: Commands,
  particle_handle: Res<ParticleTextureHandle>,
  item_handle: Res<ItemTextureHandle>,
  texture_handle: Res<TextureHandle>,
  mut assets: ResMut<Assets<Image>>,
) {
  if let Some(item_texture) = assets.get(&item_handle.0) && let Some(block_texture) = assets.get(&texture_handle.0) && let Some(particle_texture) = assets.get(&particle_handle.0) {
    let data = block_texture.data.iter().chain(item_texture.data.iter()).chain(particle_texture.data.iter()).cloned().collect();
    let array_texture = Image::new(Extent3d {
      width: item_texture.texture_descriptor.size.width,
      height: item_texture.texture_descriptor.size.height,
      depth_or_array_layers: 3,
    }, item_texture.texture_descriptor.dimension, data, item_texture.texture_descriptor.format);
    let handle = assets.add(array_texture);
    commands.insert_resource(ArrayTextureHandle(handle));
  }
}

impl Plugin for VoxelRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let voxel_shader_vertex =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/voxel.vert.spv").as_slice());
    let voxel_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../../shaders/output/voxel.frag.spv").as_slice());
    shaders.set_untracked(VOXEL_SHADER_VERTEX_HANDLE, voxel_shader_vertex);
    shaders.set_untracked(VOXEL_SHADER_FRAGMENT_HANDLE, voxel_shader_fragment);

    app.add_event::<RemeshEvent>();
    app.add_event::<RelightEvent>();
    app
      .add_plugin(ExtractResourcePlugin::<LightTextureHandle>::default())
      .init_resource::<LightTextureHandle>();

    let create_texture_array = ConditionSet::new()
      .run_if(
        move |array_texture_handle: Option<Res<ArrayTextureHandle>>,
              item_handle: Option<Res<ItemTextureHandle>>,
              texture_handle: Option<Res<TextureHandle>>| {
          item_handle.is_some() && texture_handle.is_some() && array_texture_handle.is_none()
        },
      )
      .with_system(create_texture_array_system)
      .into();

    app
      .init_resource::<TextureHandle>()
      .init_resource::<ItemTextureHandle>()
      .add_plugin::<ExtractResourcePlugin<TextureHandle>>(ExtractResourcePlugin::default())
      .add_plugin::<ExtractResourcePlugin<ItemTextureHandle>>(ExtractResourcePlugin::default())
      .add_system_set(create_texture_array);

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();

    let mut buf = OverlayBuffer {
      blocks: BufferVec::new(BufferUsages::VERTEX),
    };
    let render_device = render_app.world.resource::<RenderDevice>();
    buf.blocks.reserve(1, render_device);
    render_app.insert_resource(buf);

    render_app
      .init_resource::<ExtractedBlocks>()
      .init_resource::<VoxelPipeline>()
      .init_resource::<SpecializedRenderPipelines<VoxelPipeline>>()
      .add_system_to_stage(RenderStage::Extract, extract_chunks.run_if(in_game_extract))
      .add_system_to_stage(
        RenderStage::Extract,
        |mut commands: Commands, array_texture_handle: Extract<Option<Res<ArrayTextureHandle>>>| {
          if let Some(handle) = array_texture_handle.as_ref() {
            commands.insert_resource(ArrayTextureHandle(handle.0.clone()));
          }
        },
      )
      .add_system_to_stage(RenderStage::Queue, queue_chunks.run_if(in_game))
      .add_render_command::<Opaque3d, DrawVoxelsFull>();
  }
}
