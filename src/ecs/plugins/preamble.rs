use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::settings::WgpuSettings;
use bevy::winit::WinitWindows;

use crate::ecs::components::chunk::Chunk;
use crate::ecs::plugins::voxel::Location;

// let there: be = light;
pub struct Preamble;

impl Plugin for Preamble {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(WindowDescriptor {
        width: 1920.0,
        height: 1080.0,
        resizable: false,
        title: "仕方がない、ね？".to_string(),
        scale_factor_override: Some(1.0),
        ..default()
      })
      .insert_resource(WgpuSettings { ..default() })
      .add_system_to_stage(CoreStage::Last, exit);
    app.world.spawn().insert(Chunk::new((10, 10, 10))).insert(Location {
      x: -2.5,
      y: -6.5,
      z: -2.5,
      size_x: 5.0,
    });
  }
}

fn exit(mut events: EventReader<AppExit>, w: NonSend<WinitWindows>) {
  if w.windows.is_empty() {
    std::process::exit(0)
  }
  if let Some(_) = events.iter().next() {
    std::process::exit(0)
  }
}
