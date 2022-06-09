use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use num_traits::Pow;
use crate::ecs::components::block::Block;
use crate::ecs::components::chunk::Chunk;
use crate::ecs::plugins::voxel::RemeshEvent;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::util::array::{DDD, from_ddd};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_system_to_stage(CoreStage::PreUpdate, animation_system);
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

pub fn animation_system(
  mut animations: Query<(Entity, &mut Animation, &mut Transform)>,
  time: Res<Time>,
  mut commands: Commands,
  mut chunks: Query<&mut Chunk>,
  mut chunk_map: ResMut<ChunkMap>,
  mut remesh: EventWriter<RemeshEvent>
) {
  for (e, mut animation, mut transform) in animations.iter_mut() {
    if animation.t >= 1.0 {
      if animation.block.is_some() {
        commands.entity(e).despawn_recursive();
        let _ = std::mem::replace(chunk_map.get_mut(&mut commands, None, &mut chunks, animation.to).unwrap(), animation.block.take().unwrap());
        let free_entities = &mut chunks.get_mut(chunk_map.get_chunk_entity_or_queue(&mut commands, None, animation.from).unwrap()).unwrap().into_inner().free_entities;
        free_entities.remove(free_entities.iter().position(|x| *x == e).unwrap());
      } else {
        commands.entity(e).remove::<Animation>();
      }
      remesh.send(RemeshEvent::Remesh(ChunkMap::get_chunk_coord(animation.to)));
    } else {
      transform.translation = from_ddd(animation.from) + (from_ddd(animation.to) - from_ddd(animation.from)) * animation.t;
      animation.t += time.delta().as_secs_f32() * animation.speed;
    }
  }
}