use bevy::ecs::system::lifetimeless::{SQuery, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::pbr::{DrawMesh, MeshUniform, PreparedMaterial, RenderMaterials};
use bevy::prelude::*;
use bevy::render::render_phase::{EntityRenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass};
use bevy_atmosphere::skybox::{AtmosphereSkyBoxMaterial, SkyBoxMaterial};
use crate::ecs::plugins::rendering::draw_command::{SetBindGroup, SetTextureBindGroup, SetViewBindGroup};
use crate::ecs::plugins::rendering::mesh_pipeline::draw_command::{DrawMeshes, SetMeshDynamicBindGroup};
use crate::ecs::plugins::rendering::mesh_pipeline::systems::{MeshBuffer, PositionUniform};
use crate::ecs::plugins::rendering::skybox_pipeline::bind_groups::{SkyboxMeshPositionBindGroup, SkyboxTextureBindGroup, SkyboxViewBindGroup};
use crate::ecs::plugins::rendering::skybox_pipeline::systems::ExtractedAtmosphereSkyBoxMaterial;

pub type DrawSkyboxFull = (
  SetItemPipeline,
  SetViewBindGroup<0, SkyboxViewBindGroup>,
  SetBindGroup<1, SkyboxTextureBindGroup>,
  SetMeshDynamicBindGroup<2, SkyboxMeshPositionBindGroup, MeshUniform>,
  DrawMesh,
);