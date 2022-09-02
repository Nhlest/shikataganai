use crate::ecs::plugins::rendering::draw_command::{SetBindGroup, SetTextureBindGroup, SetViewBindGroup};
use crate::ecs::plugins::rendering::mesh_pipeline::bind_groups::{
  MeshLightBindGroup, MeshLightTextureBindGroup, MeshPositionBindGroup, MeshViewBindGroup,
};
use crate::ecs::plugins::rendering::mesh_pipeline::pipeline::RenderTextures;
use crate::ecs::plugins::rendering::mesh_pipeline::systems::{MeshBuffer, PositionUniform};
use crate::ecs::resources::light::LightLevel;
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::render_phase::{EntityRenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass};
use bevy::render::render_resource::{BindGroup, IndexFormat};
use std::marker::PhantomData;
use std::ops::Deref;
use bevy::pbr::DrawMesh;

pub type DrawMeshFull = (
  SetItemPipeline,
  SetViewBindGroup<0, MeshViewBindGroup>,
  SetTextureBindGroup<1>,
  SetMeshDynamicBindGroup<2, MeshPositionBindGroup, PositionUniform>,
  SetMeshDynamicBindGroup<3, MeshLightBindGroup, LightLevel>,
  SetBindGroup<4, MeshLightTextureBindGroup>,
  DrawMesh,
);

pub struct SetMeshDynamicBindGroup<const I: usize, B, U> {
  _b: PhantomData<B>,
  _u: PhantomData<U>,
}
impl<const I: usize, B: Send + Sync + Deref<Target = BindGroup> + 'static, U: Component> EntityRenderCommand
  for SetMeshDynamicBindGroup<I, B, U>
{
  type Param = (SRes<B>, SQuery<Read<DynamicUniformIndex<U>>>);
  #[inline]
  fn render<'w>(
    _view: Entity,
    item: Entity,
    (mesh_bind_group, mesh_query): SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let mesh_index = mesh_query.get(item).unwrap();
    pass.set_bind_group(I, mesh_bind_group.into_inner().deref(), &[mesh_index.index()]);
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
    let MeshBuffer(buf, idx_buffer, indicies, index_format) = param.get_inner(item).unwrap();
    pass.set_vertex_buffer(0, buf.slice(..));
    pass.set_index_buffer(idx_buffer.slice(..), 0, *index_format);
    pass.draw_indexed(0..*indicies as u32, 0, 0..1 as u32);
    RenderCommandResult::Success
  }
}
