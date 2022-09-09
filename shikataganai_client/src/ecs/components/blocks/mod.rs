use std::hash::Hash;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use crate::ecs::plugins::rendering::mesh_pipeline::loader::Meshes;
use crate::ecs::resources::block::BlockSprite;
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::Block;

pub mod regular_blocks;
pub mod regular_meshes;

pub enum Skeletons {
  Chest
}

#[repr(u16)]
pub enum ChestSkeleton {
  ChestBase,
  ChestLid
}

impl Skeletons {
  pub fn to_skeleton_def(&self) -> SkeletonDef {
    match self {
      Chest => {
        SkeletonDef {
          skeleton: HashMap::from([(ChestSkeleton::ChestBase as u16, Meshes::ChestBase), (ChestSkeleton::ChestLid as u16, Meshes::ChestLid)]),
          collider: Meshes::ChestBase
        }
      }
    }
  }
}

#[derive(Component)]
pub struct SkeletonAnimationFrame(pub f32);

pub struct SkeletonDef {
  pub skeleton: HashMap<u16, Meshes>,
  pub collider: Meshes
}

#[derive(Component)]
pub struct Skeleton {
  pub skeleton: HashMap<u16, Entity>
}

pub enum BlockRenderInfo {
  Nothing,
  AsBlock([BlockSprite; 6]),
  AsMesh(Meshes),
  AsSkeleton(Skeletons),
}

pub trait BlockTraitExt {
  fn render_info(&self) -> BlockRenderInfo;
}

pub trait DerefExt {
  fn deref_ext(&self) -> &dyn BlockTraitExt;
}

static BLOCK_TRAITS_EXT: [&(dyn BlockTraitExt + Sync); 7] = [
  &regular_blocks::Air,
  &regular_blocks::Dirt,
  &regular_blocks::Grass,
  &regular_blocks::Cobblestone,
  &regular_meshes::Stair,
  &regular_blocks::LightEmitter,
  &regular_meshes::Chest,
];

impl DerefExt for BlockId {
  #[inline]
  fn deref_ext(&self) -> &'static dyn BlockTraitExt {
    BLOCK_TRAITS_EXT[*self as usize]
  }
}

impl DerefExt for Block {
  #[inline]
  fn deref_ext(&self) -> &'static (dyn BlockTraitExt + '_) {
    self.block.deref_ext()
  }
}