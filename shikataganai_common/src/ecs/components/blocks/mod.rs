use crate::ecs::components::blocks::block_id::BlockId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use crate::networking::BlockTransfer;

pub mod block_id;
pub mod regular_blocks;
pub mod regular_meshes;

pub trait BlockTrait {
  fn visible(&self) -> bool {
    true
  }
  fn passable(&self) -> bool {
    false
  }
  // fn render_info(&self) -> BlockRenderInfo;
}

pub enum BlockRotation {
  NORTH,
  EAST,
  SOUTH,
  WEST,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BlockMeta {
  pub v: u32,
}

impl BlockMeta {
  pub fn get_rotation(self) -> BlockRotation {
    match self.v % 4 {
      0 => BlockRotation::NORTH,
      1 => BlockRotation::EAST,
      2 => BlockRotation::SOUTH,
      3 => BlockRotation::WEST,
      _ => panic!("Shouldn't happen"),
    }
  }
  pub fn set_rotation(&mut self, rotation: BlockRotation) {
    self.v = (self.v ^ (self.v & 0b11)) | rotation as u32;
  }
}

#[derive(Debug, Component, Copy, Clone, Serialize, Deserialize)]
pub struct Block {
  pub block: BlockId,
  pub meta: BlockMeta,
  pub entity: Entity,
}

impl Into<BlockTransfer> for Block {
  fn into(self) -> BlockTransfer {
    BlockTransfer {
      block: self.block,
      meta: self.meta
    }
  }
}

impl From<BlockTransfer> for Block {
  fn from(block: BlockTransfer) -> Self {
    Self {
      block: block.block,
      meta: block.meta,
      entity: Entity::from_bits(0)
    }
  }
}

impl Block {
  pub fn new(block: BlockId) -> Self {
    Self {
      block,
      meta: BlockMeta { v: 0 },
      entity: Entity::from_bits(0),
    }
  }
}

impl Deref for Block {
  type Target = dyn BlockTrait;

  #[inline]
  fn deref(&self) -> &Self::Target {
    self.block.deref()
  }
}
