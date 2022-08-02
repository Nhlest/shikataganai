use bevy::prelude::Deref;
use bevy::render::render_resource::BindGroup;

#[derive(Deref)]
pub struct MeshViewBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref)]
pub struct MeshTextureBindGroup {
  pub bind_group: BindGroup,
}

#[derive(Deref)]
pub struct MeshPositionBindGroup {
  pub bind_group: BindGroup,
}
