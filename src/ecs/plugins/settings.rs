use bevy::prelude::*;
use bevy::window::{PresentMode, WindowMode};
use serde::Deserialize;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Read;

pub struct SettingsPlugin;

#[derive(Deserialize, Serialize)]
pub struct Settings {
  pub sensitivity: f32,
  pub height: f32,
  pub width: f32,
  pub vsync: bool,
  pub fullscreen: bool,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      sensitivity: 1.0,
      height: 1080.0,
      width: 1920.0,
      vsync: true,
      fullscreen: false,
    }
  }
}

pub struct MouseSensitivity(pub f32);

pub struct Resolution {
  pub width: f32,
  pub height: f32,
}

pub struct VSync(pub bool);
pub struct FullScreen(pub bool);

impl VSync {
  pub fn as_present_mode(&self) -> PresentMode {
    match self.0 {
      true => PresentMode::Fifo,
      false => PresentMode::Immediate,
    }
  }
}

impl FullScreen {
  pub fn as_mode(&self) -> WindowMode {
    match self.0 {
      true => WindowMode::BorderlessFullscreen,
      false => WindowMode::Windowed,
    }
  }
}

impl Plugin for SettingsPlugin {
  fn build(&self, app: &mut App) {
    let mut file = OpenOptions::new()
      .write(true)
      .read(true)
      .create(true)
      .truncate(false)
      .open("shikataganai.toml")
      .unwrap();
    let mut str = String::new();
    file.read_to_string(&mut str).unwrap();
    let toml: Settings = toml::from_str(str.as_str()).unwrap_or_default();
    app.insert_resource(MouseSensitivity(toml.sensitivity));
    app.insert_resource(Resolution {
      width: toml.width,
      height: toml.height,
    });
    app.insert_resource(VSync(toml.vsync));
    app.insert_resource(FullScreen(toml.fullscreen));
  }
}
