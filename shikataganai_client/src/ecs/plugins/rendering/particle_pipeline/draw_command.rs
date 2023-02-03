use crate::ecs::plugins::rendering::draw_command::{SetBindGroup, SetViewBindGroup};
use crate::ecs::plugins::rendering::particle_pipeline::bind_groups::AspectRatioBindGroup;
use crate::ecs::plugins::rendering::particle_pipeline::ParticleBuffer;
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{
  LightTextureBindGroup, TextureBindGroup, ViewBindGroup,
};
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::render_phase::{EntityRenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass};

pub type DrawParticlesFull = (
  SetItemPipeline,
  SetViewBindGroup<0, ViewBindGroup>,
  SetBindGroup<1, TextureBindGroup>,
  SetBindGroup<2, AspectRatioBindGroup>,
  SetBindGroup<3, LightTextureBindGroup>,
  DrawParticles,
);

pub struct DrawParticles;
impl EntityRenderCommand for DrawParticles {
  type Param = SRes<ParticleBuffer>;

  fn render<'w>(
    _view: Entity,
    _item: Entity,
    param: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let ParticleBuffer { particles, count } = param.into_inner();
    pass.set_vertex_buffer(0, particles.buffer().unwrap().slice(..));
    pass.draw(0..4, 0..*count as u32);
    RenderCommandResult::Success
  }
}
