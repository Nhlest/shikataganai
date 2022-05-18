use crate::ecs::components::chunk::{BlockId, Chunk};
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::{Isometry3, Vector3};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::camera::CameraProjection;
use bevy::render::primitives::Frustum;
use num_traits::float::FloatConst;
use parry3d::na::Point3;
use parry3d::query;
use parry3d::query::TOIStatus::{Converged, Penetrating};
use parry3d::query::TOI;
use parry3d::shape::{Ball, Capsule, Cuboid};

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
      pos: Vec3::new(5.0, 170.0, 5.0),
      momentum: Default::default(),
    };
    let camera = {
      let perspective_projection = PerspectiveProjection {
        fov: std::f32::consts::PI / 3.0,
        near: 0.1,
        far: 1000.0,
        aspect_ratio: 1.0,
      };
      let view_projection = perspective_projection.get_projection_matrix();
      let frustum =
        Frustum::from_view_projection(&view_projection, &Vec3::ZERO, &Vec3::Z, perspective_projection.far());
      PerspectiveCameraBundle {
        camera: Camera {
          projection_matrix: perspective_projection.get_projection_matrix(),
          target: Default::default(),
          depth_calculation: Default::default(),
        },
        perspective_projection,
        frustum,
        ..default()
      }
    };
    app
      .world
      .spawn()
      .insert_bundle(camera)
      .insert(Player)
      .insert(fps_camera);
    app
      .init_resource::<Option<Selection>>()
      .add_system(movement_input_system)
      .add_system(gravity_system.after(movement_input_system))
      .add_system(collision_movement_system.after(gravity_system))
      .add_system(block_pick.after(collision_movement_system))
      .add_system(cursor_grab_system);
  }
}

fn gravity_system(mut player: Query<&mut FPSCamera>, time: Res<Time>) {
  let mut fps_camera = player.single_mut();
  fps_camera.momentum += Vec3::new(0.0, -1.0 * time.delta().as_secs_f32(), 0.0);
  fps_camera.momentum.y = fps_camera.momentum.y.clamp(-0.5, 999.0);
}

fn movement_input_system(
  mut player: Query<(&mut FPSCamera, &mut Transform)>,
  mut mouse_events: EventReader<MouseMotion>,
  key_events: Res<Input<KeyCode>>,
  mut windows: ResMut<Windows>,
  mut jumping: Local<bool>,
) {
  let window = windows.get_primary_mut().unwrap();
  if !window.cursor_locked() {
    return;
  }

  let (mut camera, transform) = player.single_mut();
  for MouseMotion { delta } in mouse_events.iter() {
    camera.phi += delta.x * 0.01;
    camera.theta = (camera.theta + delta.y * 0.01).clamp(0.05, f32::PI() - 0.05);
  }

  let mut movement = Vec3::default();
  if key_events.pressed(KeyCode::W) {
    let mut fwd = transform.forward();
    fwd.y = 0.0;
    let fwd = fwd.normalize();
    movement += fwd;
  }
  if key_events.pressed(KeyCode::A) {
    movement += transform.left()
  }
  if key_events.pressed(KeyCode::D) {
    movement += transform.right()
  }
  if key_events.pressed(KeyCode::S) {
    let mut back = transform.back();
    back.y = 0.0;
    let back = back.normalize();
    movement += back;
  }

  movement = movement.normalize_or_zero();

  if camera.momentum.y == 0.0 {
    *jumping = false;
  }
  if key_events.pressed(KeyCode::Space) && !*jumping {
    *jumping = true;
    movement.y = 10.0;
  }

  camera.momentum += movement * 0.02;
  // if camera.momentum.length() > 5.5 {
  //   camera.momentum = camera.momentum.normalize() * 1.5;
  // }
}

fn collision_movement_system(
  mut player: Query<(&mut FPSCamera, &mut Transform)>,
  chunks: Query<&Chunk>,
  chunk_map: Res<ChunkMap>,
  time: Res<Time>,
) {
  let (fps_camera, mut transform) = player.single_mut();
  let mut fps_camera: Mut<FPSCamera> = fps_camera;

  let mut aabbs = vec![];
  let cuboid = Cuboid::new(Vector3::new(0.5, 0.5, 0.5));
  let zero_vel = Vector3::new(0.0, 0.0, 0.0);
  let playeroid = Capsule::new(Point3::new(0.0, 0.3, 0.0), Point3::new(0.0, 1.5, 0.0), 0.3);
  // let playeroid = Cuboid::new(Vector3::new(0.3, 0.9, 0.3));

  for ix in -2..=2 {
    for iy in -2..=2 {
      for iz in -2..=2 {
        let c = fps_camera.pos + Vec3::new(ix as f32, iy as f32, iz as f32);
        if let Some((e, c)) = chunk_map.get_path_to_block(c) {
          if chunks.get(e).unwrap().grid[c.into()].block != BlockId::Air {
            aabbs.push(Isometry3::translation(
              c.x as f32 + 0.5,
              c.y as f32 + 0.5,
              c.z as f32 + 0.5,
            ));
          }
        }
      }
    }
  }

  for _ in 0..3 {
    let player_pos = Isometry3::translation(fps_camera.pos.x, fps_camera.pos.y - 1.5, fps_camera.pos.z);
    let player_vel = Vector3::new(fps_camera.momentum.x, fps_camera.momentum.y, fps_camera.momentum.z);
    let mut tois = vec![];

    for i in aabbs.iter() {
      match query::time_of_impact(&player_pos, &player_vel, &playeroid, &i, &zero_vel, &cuboid, 1.0) {
        Ok(f) => match f {
          None => {}
          Some(TOI {
            toi, normal2, status, ..
          }) => {
            if status == Converged {
              tois.push((toi, normal2));
            }
            if status == Penetrating {
              match query::contact(&player_pos, &playeroid, &i, &cuboid, 0.0) {
                Ok(Some(contact)) => {
                  let normal2 = contact.normal2;
                  let other = Vec3::new(normal2.x, normal2.y, normal2.z);
                  let dot = fps_camera.momentum.dot(other);
                  if dot < 0.0 {
                    fps_camera.momentum = fps_camera.momentum - dot * other;
                  }
                }
                _ => {}
              }
            }
          }
        },
        Err(e) => {
          println!("{}", e);
          std::process::exit(-1);
        }
      }
    }

    let min = tois.iter().min_by(|(x1, _), (x2, _)| x1.total_cmp(x2));
    match min {
      None => {}
      Some((toi, normal)) => {
        fps_camera.pos = fps_camera.pos + fps_camera.momentum * *toi * time.delta().as_secs_f32() * 100.0;
        let other = Vec3::new(normal.x, normal.y, normal.z);
        fps_camera.momentum = fps_camera.momentum - fps_camera.momentum.dot(other) * other;
      }
    }
  }

  fps_camera.pos = fps_camera.pos + fps_camera.momentum * time.delta().as_secs_f32() * 100.0;
  let y = fps_camera.momentum.y;
  fps_camera.momentum.y = 0.0;
  fps_camera.momentum *= 0.7; //TODO: time.delta
  fps_camera.momentum.y = y;

  transform.translation = fps_camera.pos;
  let looking_at = Vec3::new(
    10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
    10.0 * fps_camera.theta.cos(),
    10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
  ) + fps_camera.pos;
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

#[derive(Clone, Debug)]
pub struct Selection {
  pub cube: [i32; 3],
  pub face: [i32; 3],
}

fn block_pick(
  camera: Query<&Transform, With<Camera>>,
  mut selection: ResMut<Option<Selection>>,
  chunks: Query<&Chunk>,
  chunk_map: Res<ChunkMap>,
) {
  *selection = None;
  let camera_transform = camera.single();
  let camera_origin = camera_transform.translation;
  let forward = camera_transform.forward();
  let bullet = Ball::new(0.001);
  let cube = Cuboid::new(Vector3::new(0.5, 0.5, 0.5));
  let camera_origin_isometry = Isometry3::translation(camera_origin.x, camera_origin.y, camera_origin.z);
  let bullet_velocity = Vector3::new(forward.x, forward.y, forward.z);
  let zero = Vector3::new(0.0, 0.0, 0.0);

  let mut aabbs = vec![];
  for ix in -5..=5 {
    for iy in -5..=5 {
      for iz in -5..=5 {
        let c = camera_origin + Vec3::new(ix as f32, iy as f32, iz as f32);
        if let Some((e, c)) = chunk_map.get_path_to_block(c) {
          if chunks.get(e).unwrap().grid[c.into()].block != BlockId::Air {
            aabbs.push(Isometry3::translation(
              c.x as f32 + 0.5,
              c.y as f32 + 0.5,
              c.z as f32 + 0.5,
            ));
          }
        }
      }
    }
  }

  let mut tois = vec![];
  for aabb in aabbs.iter() {
    match query::time_of_impact(
      &camera_origin_isometry,
      &bullet_velocity,
      &bullet,
      aabb,
      &zero,
      &cube,
      5.0,
    ) {
      Ok(Some(TOI { toi, normal2, .. })) => tois.push((toi, normal2, aabb)),
      _ => {}
    }
  }
  match tois
    .into_iter()
    .min_by(|(toi1, _, _), (toi2, _, _)| toi1.total_cmp(toi2))
  {
    None => {}
    Some((_, normal, aabb)) => {
      *selection = Some(Selection {
        cube: [
          aabb.translation.x.floor() as i32,
          aabb.translation.y.floor() as i32,
          aabb.translation.z.floor() as i32,
        ],
        face: [
          (aabb.translation.x.floor() + normal.x.round()) as i32,
          (aabb.translation.y.floor() + normal.y.round()) as i32,
          (aabb.translation.z.floor() + normal.z.round()) as i32,
        ],
      });
    }
  }
}
