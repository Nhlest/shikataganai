use bevy::prelude::*;
use bevy::render::render_resource::BindGroup;

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
