use crate::ecs::plugins::settings::{AmbientOcclusion, FullScreen, MouseSensitivity, Resolution, Settings, VSync};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use bevy_rapier3d::prelude::RapierConfiguration;
use bevy_renet::renet::RenetClient;
use std::fs::OpenOptions;
use std::io::Write;

pub struct Preamble;

impl Plugin for Preamble {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(Msaa { samples: 1 })
      .insert_resource(RapierConfiguration {
        physics_pipeline_active: false,
        ..RapierConfiguration::default()
      })
      .add_system_to_stage(CoreStage::Last, exit);
  }
}

fn exit(
  mut events: EventReader<AppExit>,
  w: NonSend<WinitWindows>,
  sensitivity: Res<MouseSensitivity>,
  resolution: Res<Resolution>,
  vsync: Res<VSync>,
  fullscreen: Res<FullScreen>,
  ambient_occlusion: Res<AmbientOcclusion>,
  client: Option<ResMut<RenetClient>>,
) {
  if events.iter().next().is_some() || w.windows.is_empty() {
    client.map(|mut client| client.disconnect());
    let mut file = OpenOptions::new()
      .write(true)
      .create(true)
      .truncate(true)
      .open("shikataganai.toml")
      .unwrap();

    let toml = toml::to_string(&Settings {
      sensitivity: sensitivity.0,
      height: resolution.height,
      width: resolution.width,
      vsync: vsync.0,
      fullscreen: fullscreen.0,
      ambient_occlusion: ambient_occlusion.0,
    })
    .unwrap();

    file.write(toml.as_bytes()).unwrap();

    std::process::exit(0)
  }
}
