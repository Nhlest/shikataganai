pub mod mesh_pipeline;
pub mod misc;
pub mod systems;
pub mod voxel_pipeline;
pub use crate::ecs::plugins::voxel::mesh_pipeline::*;
pub use crate::ecs::plugins::voxel::misc::*;
pub use crate::ecs::plugins::voxel::systems::*;
pub use crate::ecs::plugins::voxel::voxel_pipeline::*;

use bevy::core_pipeline::Opaque3d;
use bevy::prelude::*;
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::render_resource::SpecializedRenderPipelines;
use bevy::render::{RenderApp, RenderStage};

pub struct VoxelRendererPlugin;

impl Plugin for VoxelRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let voxel_shader_vertex = Shader::from_spirv(include_bytes!("../../../../assets/shader/voxel.vert.spv").as_slice());
    let voxel_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../assets/shader/voxel.frag.spv").as_slice());
    shaders.set_untracked(VOXEL_SHADER_VERTEX_HANDLE, voxel_shader_vertex);
    shaders.set_untracked(VOXEL_SHADER_FRAGMENT_HANDLE, voxel_shader_fragment);

    app.add_event::<RemeshEvent>();
    app.add_event::<RelightEvent>();

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app
      .init_resource::<ExtractedBlocks>()
      .init_resource::<VoxelPipeline>()
      .init_resource::<SpecializedRenderPipelines<VoxelPipeline>>()
      .init_resource::<TextureHandle>()
      .init_resource::<LightTextureHandle>()
      .init_resource::<VoxelViewBindGroup>()
      .init_resource::<SelectionBindGroup>()
      .init_resource::<VoxelTextureBindGroup>()
      .init_resource::<LightTextureBindGroup>()
      .add_system_to_stage(RenderStage::Extract, extract_chunks)
      .add_system_to_stage(RenderStage::Queue, queue_chunks)
      .add_render_command::<Opaque3d, DrawVoxelsFull>();
  }
}
