use crate::ecs::components::block_or_item::BlockOrItem;
use crate::ecs::components::blocks::block_id::BlockId;

#[derive(Default)]
pub struct SelectedHotBar(pub i32);

#[derive(Default)]
pub struct RerenderInventory(pub bool);

pub struct QuantifiedBlockOrItem {
  pub block_or_item: BlockOrItem,
  pub quant: u32,
}

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
        None,
        None,
        None,
        None,
      ],
    }
  }
}
