use crate::ecs::plugins::rendering::draw_command::{SetBindGroup, SetViewBindGroup};
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{
  LightTextureBindGroup, SelectionBindGroup, TextureBindGroup, ViewBindGroup,
};
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::ChunkMeshBuffer;
use bevy::ecs::system::lifetimeless::{Read, SQuery};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::render_phase::{EntityRenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass};

pub type DrawVoxelsFull = (
  SetItemPipeline,
  SetViewBindGroup<0, ViewBindGroup>,
  SetBindGroup<1, TextureBindGroup>,
  SetBindGroup<2, SelectionBindGroup>,
  SetBindGroup<3, LightTextureBindGroup>,
  DrawVoxels,
);

pub struct DrawVoxels;
impl EntityRenderCommand for DrawVoxels {
  type Param = SQuery<Read<ChunkMeshBuffer>>;

  fn render<'w>(
    _view: Entity,
    item: Entity,
    param: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let ChunkMeshBuffer(buf, verticies) = param.get_inner(item).unwrap();
    pass.set_vertex_buffer(0, buf.slice(..));
    pass.draw(0..*verticies as u32 * 6, 0..1_u32);
    RenderCommandResult::Success
  }
}
