use crate::ecs::resources::ui::UiSprite;
use crate::ecs::components::block_or_item::BlockOrItem;

#[derive(Default)]
pub struct SelectedHotBar(pub i32);

pub struct PlayerInventory {
  pub items: Vec<BlockOrItem>,
}

impl Default for PlayerInventory {
  fn default() -> Self {
    Self {
      items: vec![
        BlockOrItem::Empty,
        BlockOrItem::Empty,
        BlockOrItem::Empty
      ],
    }
  }
}
