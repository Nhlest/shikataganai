use bevy::prelude::*;
use bevy::render::render_resource::BindGroup;

#[derive(Deref, Resource)]
pub struct AspectRatioBindGroup {
  pub bind_group: BindGroup,
}
