use crate::ecs::plugins::camera::Selection;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::winit::WinitWindows;

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
      .insert_resource(Msaa { samples: 1 })
      .init_resource::<Option<Selection>>()
      .add_system_to_stage(CoreStage::Last, exit);
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
