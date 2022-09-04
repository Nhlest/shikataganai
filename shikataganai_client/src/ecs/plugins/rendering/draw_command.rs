use crate::ecs::plugins::rendering::mesh_pipeline::pipeline::RenderTextures;
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::render_phase::{
  EntityRenderCommand, PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass,
};
use bevy::render::render_resource::BindGroup;
use bevy::render::view::ViewUniformOffset;
use std::marker::PhantomData;
use std::ops::Deref;

pub struct SetBindGroup<const I: usize, T: Deref<Target = BindGroup> + Send + Sync + 'static> {
  _phantom: PhantomData<T>,
}
impl<P: PhaseItem, const I: usize, T: Deref<Target = BindGroup> + Send + Sync + 'static> RenderCommand<P>
  for SetBindGroup<I, T>
{
  type Param = SRes<T>;

  fn render<'w>(
    _view: Entity,
    _item: &P,
    bind_group: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let texture_bind_group = bind_group.into_inner().deref();
    pass.set_bind_group(I, texture_bind_group, &[]);
    RenderCommandResult::Success
  }
}

pub struct SetViewBindGroup<const I: usize, T: Deref<Target = BindGroup> + Send + Sync + 'static> {
  _phantom: PhantomData<T>,
}
impl<P: PhaseItem, const I: usize, T: Deref<Target = BindGroup> + Send + Sync + 'static> RenderCommand<P>
  for SetViewBindGroup<I, T>
{
  type Param = (SRes<T>, SQuery<Read<ViewUniformOffset>>);

  fn render<'w>(
    view: Entity,
    _item: &P,
    (bind_group, view_query): SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let view_uniform = view_query.get(view).unwrap();
    let bind_group = bind_group.into_inner().deref();
    pass.set_bind_group(I, bind_group, &[view_uniform.offset]);
    RenderCommandResult::Success
  }
}

pub struct SetTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetTextureBindGroup<I> {
  type Param = (SRes<RenderTextures>, SQuery<Read<Handle<Image>>>);
  fn render<'w>(
    _view: Entity,
    item: Entity,
    (textures, query): SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let texture_handle = query.get(item).unwrap();
    let texture = textures.into_inner().get(texture_handle).unwrap();
    pass.set_bind_group(I, &texture, &[]);
    RenderCommandResult::Success
  }
}
