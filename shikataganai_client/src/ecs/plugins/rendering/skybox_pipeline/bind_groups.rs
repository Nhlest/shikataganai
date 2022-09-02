use bevy::render::render_resource::BindGroup;
use bevy::prelude::*;

#[derive(Deref)]
pub struct SkyboxViewBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref)]
pub struct SkyboxTextureBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref)]
pub struct SkyboxMeshPositionBindGroup {
  pub bind_group: BindGroup,
}
