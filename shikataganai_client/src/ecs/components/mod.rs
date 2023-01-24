pub mod blocks;
pub mod items;

use crate::ecs::components::blocks::BlockSprite;
use bevy::ecs::component::Component;

#[derive(Component)]
pub struct OverlayRender {
  pub overlays: [BlockSprite; 6],
}
