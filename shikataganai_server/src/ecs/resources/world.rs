use bevy::utils::hashbrown::HashMap;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::util::array::DD;

pub struct World {
  pub chunk_map: HashMap<DD, Chunk>,
}
