use crate::ecs::components::block_or_item::BlockOrItem;

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
      items: vec![None, None],
    }
  }
}
