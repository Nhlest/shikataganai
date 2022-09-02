#[derive(Copy, Clone, Debug)]
pub struct LightLevel {
  pub heaven: u8,
  pub hearth: u8,
}

impl LightLevel {
  pub fn new(heaven: u8, hearth: u8) -> Self {
    Self { heaven, hearth }
  }
  pub fn dark() -> Self {
    Self { heaven: 0, hearth: 0 }
  }
}