use bevy::render::render_resource::BindGroup;
use bevy::prelude::*;

#[derive(Deref, Resource)]
pub struct AspectRatioBindGroup {
  pub bind_group: BindGroup,
}
