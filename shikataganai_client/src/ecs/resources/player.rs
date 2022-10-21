use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::{BlockOrItem, QuantifiedBlockOrItem};

#[derive(Default)]
pub struct SelectedHotBar(pub i32);

#[derive(Default)]
pub struct RerenderInventory(pub bool);

pub struct PlayerInventory {
  pub items: Vec<Option<QuantifiedBlockOrItem>>,
}

impl Default for PlayerInventory {
  fn default() -> Self {
    Self {
      items: vec![
        Some(QuantifiedBlockOrItem {
          block_or_item: BlockOrItem::Block(BlockId::Stair),
          quant: 100,
        }),
        Some(QuantifiedBlockOrItem {
          block_or_item: BlockOrItem::Block(BlockId::LightEmitter),
          quant: 100,
        }),
        Some(QuantifiedBlockOrItem {
          block_or_item: BlockOrItem::Block(BlockId::Chest),
          quant: 100,
        }),
        None,
        None,
        None,
        None,
      ],
    }
  }
}
