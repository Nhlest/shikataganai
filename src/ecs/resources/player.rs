use crate::ecs::resources::ui::UiSprite;

#[derive(Default)]
pub struct SelectedHotBar(pub i32);

#[derive(Debug)]
pub enum HotBarItem {
  PushPull,
  HoistUnhoist,
  Delete,
  Empty,
}

impl HotBarItem {
  pub fn sprite(&self) -> UiSprite {
    match self {
      HotBarItem::PushPull => UiSprite::PushPull,
      HotBarItem::HoistUnhoist => UiSprite::HoistUnhoist,
      HotBarItem::Delete => UiSprite::Delete,
      HotBarItem::Empty => UiSprite::Empty,
    }
  }
}

pub struct HotBarItems {
  pub items: Vec<HotBarItem>,
}

impl Default for HotBarItems {
  fn default() -> Self {
    Self {
      items: vec![
        HotBarItem::PushPull,
        HotBarItem::HoistUnhoist,
        HotBarItem::Delete,
        HotBarItem::Empty,
        HotBarItem::Empty,
        HotBarItem::Empty,
      ],
    }
  }
}
