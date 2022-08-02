use crate::ecs::plugins::rendering::draw_command::{SetBindGroup, SetViewBindGroup};
use crate::ecs::plugins::rendering::mesh_pipeline::bind_groups::{
  MeshPositionBindGroup, MeshTextureBindGroup, MeshViewBindGroup,
};
use crate::ecs::plugins::rendering::mesh_pipeline::systems::{MeshBuffer, PositionUniform};
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::render_phase::{EntityRenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass};
use bevy::render::render_resource::IndexFormat;

pub type DrawMeshFull = (
  SetItemPipeline,
  SetViewBindGroup<0, MeshViewBindGroup>,
  SetBindGroup<1, MeshTextureBindGroup>,
  SetMeshPositionBindGroup<2>,
  DrawMeshes,
);

pub struct SetMeshPositionBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetMeshPositionBindGroup<I> {
  type Param = (
    SRes<MeshPositionBindGroup>,
    SQuery<Read<DynamicUniformIndex<PositionUniform>>>,
  );
  #[inline]
  fn render<'w>(
    _view: Entity,
    item: Entity,
    (mesh_bind_group, mesh_query): SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let mesh_index = mesh_query.get(item).unwrap();
    pass.set_bind_group(I, &mesh_bind_group.into_inner(), &[mesh_index.index()]);
    RenderCommandResult::Success
  }
}

pub struct DrawMeshes;
impl EntityRenderCommand for DrawMeshes {
  type Param = SQuery<Read<MeshBuffer>>;

  fn render<'w>(
    _view: Entity,
    item: Entity,
    param: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let MeshBuffer(buf, idx_buffer, indicies) = param.get_inner(item).unwrap();
    pass.set_vertex_buffer(0, buf.slice(..));
    pass.set_index_buffer(idx_buffer.slice(..), 0, IndexFormat::Uint32);
    pass.draw_indexed(0..*indicies as u32, 0, 0..1 as u32);
    RenderCommandResult::Success
  }
}
