pub mod blocks;
pub mod items;

use bevy::ecs::component::Component;

#[derive(Component)]
pub struct AnimatedThingamabob {
  pub state: i32
}