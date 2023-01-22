use crate::ecs::components::items::{ItemSprite, ItemTraitExt};

pub struct Coal;
pub struct Wand;
pub struct Iron;

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

impl ItemTraitExt for Iron {
  fn render_info(&self) -> ItemSprite {
    ItemSprite::Iron
  }
}
