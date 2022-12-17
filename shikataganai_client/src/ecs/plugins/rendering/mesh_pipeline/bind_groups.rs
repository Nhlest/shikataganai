use bevy::prelude::{Deref, Resource};
use bevy::render::render_resource::BindGroup;

#[derive(Deref, Resource)]
pub struct MeshViewBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref, Resource)]
pub struct MeshTextureBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref, Resource)]
pub struct MeshPositionBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref, Resource)]
pub struct MeshLightBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref, Resource)]
pub struct MeshLightTextureBindGroup {
  pub bind_group: BindGroup,
}
