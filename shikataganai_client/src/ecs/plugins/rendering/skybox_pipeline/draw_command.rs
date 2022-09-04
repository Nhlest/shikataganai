use crate::ecs::plugins::rendering::draw_command::{SetBindGroup, SetViewBindGroup};
use crate::ecs::plugins::rendering::mesh_pipeline::draw_command::SetMeshDynamicBindGroup;
use crate::ecs::plugins::rendering::skybox_pipeline::bind_groups::{
  SkyboxMeshPositionBindGroup, SkyboxTextureBindGroup, SkyboxViewBindGroup,
};
use bevy::pbr::{DrawMesh, MeshUniform};
use bevy::render::render_phase::SetItemPipeline;

pub type DrawSkyboxFull = (
  SetItemPipeline,
  SetViewBindGroup<0, SkyboxViewBindGroup>,
  SetBindGroup<1, SkyboxTextureBindGroup>,
  SetMeshDynamicBindGroup<2, SkyboxMeshPositionBindGroup, MeshUniform>,
  DrawMesh,
);
