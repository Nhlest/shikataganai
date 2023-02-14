use crate::ecs::plugins::camera::Player;
use crate::ecs::plugins::client::spawn_client;
use crate::ecs::plugins::game::ShikataganaiGameState;
use crate::ecs::plugins::settings::{AmbientOcclusion, FullScreen, MouseSensitivity, RecentConnections, Resolution, VSync};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::*;
use egui::{Align, emath, Layout, Widget};
use iyes_loopless::state::NextState;
use shikataganai_server::ecs::plugins::server::ShikataganaiServerAddress;
use shikataganai_server::spawn_server;
use std::ops::{DerefMut, RangeInclusive};

#[derive(Default)]
pub struct LocalString<const T: &'static str>(pub String);

pub fn main_menu(
  mut commands: Commands,
  mut egui: ResMut<EguiContext>,
  mut window: ResMut<Windows>,
  mut settings_menu_opened: Local<bool>,
  mut app_exit: EventWriter<AppExit>,
  mut mouse_sensetivity: ResMut<MouseSensitivity>,
  mut resolution: ResMut<Resolution>,
  mut vsync: ResMut<VSync>,
  mut fullscreen: ResMut<FullScreen>,
  mut recent_connections: ResMut<RecentConnections>,
  mut ambient_occlusion: ResMut<AmbientOcclusion>,
  mut address_string: Local<LocalString<"IP">>,
  mut nickname_string: Local<LocalString<"Nickname">>,
  player_entity: Query<Entity, With<Player>>,
) {
  const MAIN_MENU_SIZE : (f32, f32) = (500.0, 400.0);
  let x = (window.primary().width() - MAIN_MENU_SIZE.0) / 2.0;
  let y = 200.0;
  let player_entity = player_entity.single();
  egui::Window::new("Main Menu").fixed_pos((x, y)).fixed_size(MAIN_MENU_SIZE).show(egui.ctx_mut(), |ui| {
    ui.allocate_space(emath::Vec2::new(MAIN_MENU_SIZE.0, 0.0));
    let address = if address_string.0.is_empty() {
      "127.0.0.1:8181".to_string()
    } else {
      address_string.0.clone()
    };
    let nickname = if nickname_string.0.is_empty() {
      "Player".to_string()
    } else {
      nickname_string.0.clone()
    };
    if ui.button("Connect").clicked() {
      if !recent_connections.0.contains(&(address.clone(), nickname.clone())) {
        recent_connections.0.push((address.clone(), nickname.clone()));
      }
      commands.insert_resource(NextState(ShikataganaiGameState::PreSimulation));
      spawn_client(&mut commands, player_entity, address.clone(), nickname);
    }

    let mut to_delete = vec![];
    for (i, (ip, nick)) in recent_connections.0.iter().enumerate() {
      ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        if egui::widgets::Link::new(format!("{} @ {}", nick, ip)).ui(ui).clicked() {
          commands.insert_resource(NextState(ShikataganaiGameState::PreSimulation));
          spawn_client(&mut commands, player_entity, ip.clone(), nick.clone());
        }
        if egui::Link::new("Local Server").ui(ui).clicked() {
          let address = ip.clone();
          std::thread::spawn(|| {
            spawn_server(ShikataganaiServerAddress { address });
          });
          commands.insert_resource(NextState(ShikataganaiGameState::PreSimulation));
          spawn_client(&mut commands, player_entity, ip.clone(), nick.clone());
        }
        if egui::Link::new("Delete").ui(ui).clicked() {
          to_delete.push(i);
        }
      });
    }
    to_delete.iter().copied().for_each(|i| {recent_connections.0.remove(i);});

    egui::TextEdit::singleline(&mut address_string.deref_mut().0)
      .desired_width(MAIN_MENU_SIZE.0)
      .hint_text("127.0.0.1:8181")
      .show(ui);
    egui::TextEdit::singleline(&mut nickname_string.deref_mut().0)
      .desired_width(MAIN_MENU_SIZE.0)
      .hint_text("Player")
      .show(ui);

    if ui.button("Start Server").clicked() {
      std::thread::spawn(|| {
        spawn_server(ShikataganaiServerAddress { address });
      });
    }
    if ui.button("Settings").clicked() {
      *settings_menu_opened = true;
    }
    if ui.button("Exit").clicked() {
      app_exit.send(AppExit)
    }
  });
  if !*settings_menu_opened {
    return;
  }
  egui::Window::new("Settings Menu").show(egui.ctx_mut(), |ui| {
    egui::Slider::new(&mut mouse_sensetivity.as_mut().0, RangeInclusive::new(0.0, 2.0)).ui(ui);
    let selected = format!("{}x{}", resolution.width as i32, resolution.height as i32);
    let mut selection = (resolution.width as i32, resolution.height as i32);
    if egui::ComboBox::from_label("Resolution")
      .selected_text(selected)
      .show_ui(ui, |ui| {
        for s in [
          (320, 180),
          (640, 360),
          (853, 480),
          (1280, 720),
          (1920, 1080),
          (2560, 1444),
          (3840, 2160),
        ] {
          ui.selectable_value(&mut selection, s, format!("{:?}", s));
        }
      })
      .response
      .clicked()
    {
      //TODO: doesnt really work
      resolution.width = selection.0 as f32;
      resolution.height = selection.1 as f32;
      window
        .get_primary_mut()
        .unwrap()
        .set_resolution(resolution.width, resolution.height);
    }
    if ui.checkbox(&mut vsync.as_mut().0, "Vi Sink").changed() {
      window
        .get_primary_mut()
        .unwrap()
        .set_present_mode(vsync.as_present_mode());
    }
    if ui.checkbox(&mut fullscreen.as_mut().0, "Fullscreen").changed() {
      window.get_primary_mut().unwrap().set_mode(fullscreen.as_mode());
    }
    ui.checkbox(&mut ambient_occlusion.as_mut().0, "Ambient Occlusion");
    if ui.button("Close").clicked() {
      *settings_menu_opened = false;
    }
  });
}
