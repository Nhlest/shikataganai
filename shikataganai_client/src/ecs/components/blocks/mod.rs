use crate::ecs::plugins::rendering::mesh_pipeline::loader::Meshes;
use crate::ecs::resources::block::BlockSprite;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy_renet::renet::RenetClient;
use num_traits::FloatConst;
use shikataganai_common::ecs::components::blocks::animation::{Animation, AnimationType};
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::Block;
use shikataganai_common::util::array::DDD;

pub mod regular_blocks;
pub mod regular_meshes;

pub enum Skeletons {
  Chest,
}

#[repr(u16)]
pub enum ChestSkeleton {
  ChestBase,
  ChestLid,
}

impl Skeletons {
  pub fn to_skeleton_def(&self) -> SkeletonDef {
    match self {
      Skeletons::Chest => SkeletonDef {
        skeleton: HashMap::from([
          (
            ChestSkeleton::ChestBase as u16,
            SkeletonBoneDef::no_offset(Meshes::ChestBase),
          ),
          (
            ChestSkeleton::ChestLid as u16,
            SkeletonBoneDef::new(Meshes::ChestLid, Vec3::new(7.0 / 16.0, 2.0 / 16.0, 0.0)),
          ),
        ]),
        collider: Meshes::ChestBase,
      },
    }
  }
}

pub fn animate(commands: &mut Commands, entity: Entity, animation: Animation) {
  commands.entity(entity).insert(AnimationInstance { animation, t: 0.0 });
}

pub struct SkeletonDef {
  pub skeleton: HashMap<u16, SkeletonBoneDef>,
  pub collider: Meshes,
}

pub struct SkeletonBoneDef {
  pub mesh: Meshes,
  pub offset: Vec3,
}

impl SkeletonBoneDef {
  pub fn new(mesh: Meshes, offset: Vec3) -> Self {
    Self { mesh, offset }
  }

  pub fn no_offset(mesh: Meshes) -> Self {
    Self {
      mesh,
      offset: Vec3::ZERO,
    }
  }
}

#[derive(Component)]
pub struct Skeleton {
  pub skeleton: HashMap<u16, Entity>,
}

pub enum ChestAnimations {
  Open,
  Close,
}

#[derive(Component)]
pub struct AnimationInstance {
  pub animation: Animation,
  pub t: f32,
}

pub trait AnimationTrait {
  fn get_animation(&self) -> Animation;
}

impl AnimationTrait for ChestAnimations {
  fn get_animation(&self) -> Animation {
    let closed = Quat::from_rotation_z(0.0);
    let opened = Quat::from_rotation_z(-f32::FRAC_PI_2());
    match self {
      ChestAnimations::Open => Animation {
        animation: AnimationType::LinearRotation {
          from: closed,
          to: opened,
        },
        bone: ChestSkeleton::ChestLid as u16,
        duration: 0.5,
      },
      ChestAnimations::Close => Animation {
        animation: AnimationType::LinearRotation {
          from: opened,
          to: closed,
        },
        bone: ChestSkeleton::ChestLid as u16,
        duration: 0.5,
      },
    }
  }
}

pub enum BlockRenderInfo {
  Nothing,
  AsBlock([BlockSprite; 6]),
  AsMesh(Meshes),
  AsSkeleton(Skeletons),
}

pub trait BlockTraitExt {
  fn render_info(&self) -> BlockRenderInfo;
  fn right_click_interface(
    &self,
    _entity: Entity,
    _location: DDD,
    _commands: &mut Commands,
    _client: &mut RenetClient,
  ) -> Option<()> {
    None
  }
}

pub trait DerefExt {
  fn deref_ext(&self) -> &'static dyn BlockTraitExt;
}

static BLOCK_TRAITS_EXT: [&(dyn BlockTraitExt + Sync); 8] = [
  &regular_blocks::Air,
  &regular_blocks::Dirt,
  &regular_blocks::Grass,
  &regular_blocks::Cobblestone,
  &regular_blocks::Iron,
  &regular_meshes::Stair,
  &regular_meshes::Chest,
  &regular_blocks::Furnace,
];

impl DerefExt for BlockId {
  #[inline]
  fn deref_ext(&self) -> &'static dyn BlockTraitExt {
    BLOCK_TRAITS_EXT[*self as usize]
  }
}

impl DerefExt for Block {
  #[inline]
  fn deref_ext(&self) -> &'static dyn BlockTraitExt {
    self.block.deref_ext()
  }
}
