use bevy::prelude::*;

#[derive(Component, Clone, Resource)]
pub struct PlayerNickname(pub String);
