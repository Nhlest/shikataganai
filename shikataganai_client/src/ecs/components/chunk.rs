use crate::ecs::resources::light::LightLevel;
use bevy::prelude::*;
use bevy::tasks::Task;
use noise::*;
use shikataganai_common::ecs::components::blocks::block_id::BlockId;
use shikataganai_common::ecs::components::blocks::Block;
use shikataganai_common::ecs::components::chunk::Chunk;
use shikataganai_common::util::array::{Array, Array2d, Array3d, Bounds, DD, DDD};

