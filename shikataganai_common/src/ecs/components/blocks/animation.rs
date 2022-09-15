use bevy::prelude::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AnimationType {
  LinearMovement {
    from: Vec3,
    to: Vec3
  },
  LinearRotation {
    from: Quat,
    to: Quat
  },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Animation {
  pub animation: AnimationType,
  pub bone: u16,
  pub duration: f32
}
