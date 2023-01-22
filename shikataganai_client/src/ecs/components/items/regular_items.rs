use crate::ecs::components::items::{ItemSprite, ItemTraitExt};

pub struct Coal;
pub struct Wand;

impl ItemTraitExt for Coal {
  fn render_info(&self) -> ItemSprite {
    ItemSprite::Coal
  }
}

impl ItemTraitExt for Wand {
  fn render_info(&self) -> ItemSprite {
    ItemSprite::Wand
  }
}
