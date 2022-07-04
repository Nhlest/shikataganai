use bevy::prelude::*;
use crate::ecs::components::block::Block;
use crate::ecs::components::chunk::Chunk;
use crate::ecs::plugins::voxel::RemeshEvent;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::util::array::{DDD, from_ddd};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
  fn build(&self, app: &mut App) {
    // app
    //   .add_system_to_stage(CoreStage::PreUpdate, animation_system);
  }
}

pub enum AnimationType {
  Linear,
}

#[derive(Component)]
pub struct Animation {
  animation_type: AnimationType,
  pub block: Option<Block>,
  from: DDD,
  to: DDD,
  t: f32,
  speed: f32
}

impl Animation {
  pub fn new(from: DDD, to: DDD, speed: f32, block: Option<Block>) -> Self {
    Self {
      animation_type: AnimationType::Linear,
      block,
      from,
      to,
      t: 0.0,
      speed
    }
  }
}