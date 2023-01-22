use crate::ecs::components::blocks::block_id::BlockId;
use crate::ecs::components::item::ItemId;
use crate::recipes::SimpleRecipe;
use crate::util::array::Array;

pub fn populate_in_world_recipes() -> [SimpleRecipe; 2] {
  [
    SimpleRecipe {
      from: Array::new_init(((0, 0, 0), (1, 1, 1)), |_| BlockId::Cobble),
      to: Array::new_init(((0, 0, 0), (1, 1, 1)), |x| {
        if x == (0, 0, 0) {
          BlockId::Furnace
        } else {
          BlockId::Air
        }
      }),
      item: None,
    },
    SimpleRecipe {
      from: Array::new_init(((0, 0, 0), (0, 1, 0)), |x| {
        if x.1 == 0 {
          BlockId::Furnace
        } else {
          BlockId::Iron
        }
      }),
      to: Array::new_init(((0, 0, 0), (0, 1, 0)), |x| {
        if x == (0, 0, 0) {
          BlockId::Furnace
        } else {
          BlockId::Air
        }
      }),
      item: Some(ItemId::Iron),
    },
  ]
}
