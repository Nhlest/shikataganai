use crate::ecs::components::blocks::QuantifiedBlockOrItem;

pub struct InternalInventory {
  inventory: Vec<QuantifiedBlockOrItem>
}

pub enum FunctorTransit {
  InternalInventory(Vec<QuantifiedBlockOrItem>)
}