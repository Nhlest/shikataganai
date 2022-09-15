use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::components::blocks::{BlockOrItem, QuantifiedBlockOrItem};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Serialize, Deserialize)]
pub struct InternalInventory {
  pub inventory: Vec<Option<QuantifiedBlockOrItem>>,
}

impl InternalInventory {
  pub fn with_capacity(len: usize) -> Self {
    Self {
      inventory: (0..len).map(|_| Some(QuantifiedBlockOrItem{ block_or_item: BlockOrItem::Block(BlockId::Cobble), quant: 5 })).collect(),
    }
  }
}

pub enum FunctorTransit {
  InternalInventory(Vec<QuantifiedBlockOrItem>),
}
