const BLOCK_SPRITE_SHEET_WIDTH: usize = 8;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum BlockSprite {
  Empty,
  Dirt,
  HalfGrass,
  Grass,
  Cobblestone,
}

impl BlockSprite {
  pub const fn into_uv(self) -> ([f32; 2], [f32; 2]) {
    let i = self as usize;
    let x = i % BLOCK_SPRITE_SHEET_WIDTH;
    let y = i / BLOCK_SPRITE_SHEET_WIDTH;
    (
      [
        x as f32 / BLOCK_SPRITE_SHEET_WIDTH as f32,
        y as f32 / BLOCK_SPRITE_SHEET_WIDTH as f32,
      ],
      [
        (x + 1) as f32 / BLOCK_SPRITE_SHEET_WIDTH as f32,
        (y + 1) as f32 / BLOCK_SPRITE_SHEET_WIDTH as f32,
      ],
    )
  }
}
