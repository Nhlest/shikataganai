use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::{BlockOrItem, QuantifiedBlockOrItem};

#[derive(Default)]
pub struct SelectedHotBar(pub i32);

pub struct PlayerInventory {
  pub hot_bar_width: usize,
  pub items: Vec<Option<QuantifiedBlockOrItem>>,
}

impl Default for PlayerInventory {
  fn default() -> Self {
    Self {
      hot_bar_width: 9,
      items: vec![
        Some(QuantifiedBlockOrItem {
          block_or_item: BlockOrItem::Block(BlockId::Stair),
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
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(QuantifiedBlockOrItem {
          block_or_item: BlockOrItem::Block(BlockId::Grass),
          quant: 25
        }),
        None,
        None,
        None,
        None,
        None,
      ],
    }
  }
}
