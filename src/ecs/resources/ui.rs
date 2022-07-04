pub const UI_SPRITE_SHEET_WIDTH: usize = 4;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum UiSprite {
  PushPull,
  HoistUnhoist,
  Delete,
  Empty,
  BlueSquare,
  RedSquare,
}

impl UiSprite {
  pub const fn into_uv(self) -> ([f32; 2], [f32; 2]) {
    let i = self as usize;
    let x = i % UI_SPRITE_SHEET_WIDTH;
    let y = i / UI_SPRITE_SHEET_WIDTH;
    return (
      [
        x as f32 / UI_SPRITE_SHEET_WIDTH as f32,
        y as f32 / UI_SPRITE_SHEET_WIDTH as f32,
      ],
      [
        (x + 1) as f32 / UI_SPRITE_SHEET_WIDTH as f32,
        (y + 1) as f32 / UI_SPRITE_SHEET_WIDTH as f32,
      ],
    );
  }
}
