use shikataganai_common::ecs::components::item::ItemId;

pub mod regular_items;

const ITEM_SPRITE_SHEET_WIDTH: usize = 8;

pub enum ItemSprite {
  Nothing,
  Coal,
  Wand,
  Iron
}

impl ItemSprite {
  pub const fn into_uv(self) -> ([f32; 2], [f32; 2]) {
    let i = self as usize;
    let x = i % ITEM_SPRITE_SHEET_WIDTH;
    let y = i / ITEM_SPRITE_SHEET_WIDTH;
    (
      [
        x as f32 / ITEM_SPRITE_SHEET_WIDTH as f32,
        y as f32 / ITEM_SPRITE_SHEET_WIDTH as f32,
      ],
      [
        (x + 1) as f32 / ITEM_SPRITE_SHEET_WIDTH as f32,
        (y + 1) as f32 / ITEM_SPRITE_SHEET_WIDTH as f32,
      ],
    )
  }
}

pub trait ItemTraitExt {
  fn render_info(&self) -> ItemSprite;
}

pub trait ItemDerefExt {
  fn deref_ext(&self) -> &dyn ItemTraitExt;
}

static ITEM_TRAITS_EXT: [&(dyn ItemTraitExt + Sync); 3] = [
  &regular_items::Coal,
  &regular_items::Wand,
  &regular_items::Iron,
];

impl ItemDerefExt for ItemId {
  #[inline]
  fn deref_ext(&self) -> &'static dyn ItemTraitExt {
    ITEM_TRAITS_EXT[*self as usize]
  }
}