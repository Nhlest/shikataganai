pub struct Restrain(pub bool);

impl Restrain {
  pub fn restrain(&mut self) {
    self.0 = true;
  }
}
