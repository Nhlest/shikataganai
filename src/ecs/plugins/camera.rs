use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use num_traits::float::FloatConst;

pub struct CameraPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FPSCamera {
  phi: f32,
  theta: f32,
  pos: Vec3,
  momentum: Vec3,
}

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    let fps_camera = FPSCamera {
      phi: 0.0,
      theta: 0.0,
      pos: Default::default(),
      momentum: Default::default(),
    };
    let camera = PerspectiveCameraBundle::new_3d();
    app
      .world
      .spawn()
      .insert_bundle(camera)
      .insert(Player)
      .insert(fps_camera);
    app.add_system(fps_camera_system).add_system(cursor_grab_system);
  }
}

fn fps_camera_system(
  mut player: Query<(&mut FPSCamera, &mut Transform)>,
  mut mouse_events: EventReader<MouseMotion>,
  key_events: Res<Input<KeyCode>>,
  mut windows: ResMut<Windows>,
) {
  let window = windows.get_primary_mut().unwrap();
  if !window.cursor_locked() {
    return;
  }

  let (mut camera, mut transform) = player.single_mut();
  for MouseMotion { delta } in mouse_events.iter() {
    camera.phi += delta.x * 0.01;
    camera.theta = (camera.theta + delta.y * 0.01).clamp(0.05, f32::PI() - 0.05);
  }

  let mut movement = Vec3::default();
  if key_events.pressed(KeyCode::W) {
    movement += transform.forward()
  }
  if key_events.pressed(KeyCode::A) {
    movement += transform.left()
  }
  if key_events.pressed(KeyCode::D) {
    movement += transform.right()
  }
  if key_events.pressed(KeyCode::S) {
    movement += transform.back()
  }

  movement.y = 0.0;
  let movement = movement.normalize_or_zero();
  camera.momentum += movement * 0.02;
  if camera.momentum.length() > 0.5 {
    camera.momentum = camera.momentum.normalize() * 0.5;
  }
  camera.pos = camera.pos + camera.momentum;
  camera.momentum *= 0.7;

  transform.translation = camera.pos;
  let looking_at = Vec3::new(
    10.0 * camera.phi.cos() * camera.theta.sin(),
    10.0 * camera.theta.cos(),
    10.0 * camera.phi.sin() * camera.theta.sin(),
  );
  transform.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));
}

fn cursor_grab_system(mut windows: ResMut<Windows>, btn: Res<Input<MouseButton>>, key: Res<Input<KeyCode>>) {
  let window = windows.get_primary_mut().unwrap();

  if btn.just_pressed(MouseButton::Left) {
    window.set_cursor_lock_mode(true);
    window.set_cursor_visibility(false);
  }

  if key.just_pressed(KeyCode::Escape) {
    window.set_cursor_lock_mode(false);
    window.set_cursor_visibility(true);
  }
}
