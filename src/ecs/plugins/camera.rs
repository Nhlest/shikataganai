use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::camera::CameraProjection;
use bevy::render::primitives::{Aabb, Frustum};
use bevy::render::view::VisibleEntities;
use num_traits::float::FloatConst;
use num_traits::{Float, Signed};
use parry3d::math::Real;
use parry3d::na::{Point, Point3};
use parry3d::query;
use parry3d::query::{Contact, TOI, TOIStatus, Unsupported};
use parry3d::query::TOIStatus::{Converged, Penetrating};
use parry3d::shape::{Capsule, Cuboid};
use crate::ecs::components::chunk::{Block, BlockId, Chunk};
use crate::ecs::resources::chunk_map::{ChunkMap, ChunkMeta};
use crate::{Isometry3, Vector3};

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
      let frustum = Frustum::from_view_projection(
        &view_projection,
        &Vec3::ZERO,
        &Vec3::Z,
        perspective_projection.far(),
      );
      PerspectiveCameraBundle {
        camera: Camera {
          near: perspective_projection.near,
          far: perspective_projection.far,
          ..Default::default()
        },
        perspective_projection,
        frustum,
        .. default()
      }
    };
    app
      .world
      .spawn()
      .insert_bundle(camera)
      .insert(Player)
      .insert(fps_camera);
    app
      .add_system(movement_input_system)
      .add_system(gravity_system.after(movement_input_system))
      .add_system(collision_movement_system.after(gravity_system))
      .add_system(cursor_grab_system);
  }
}

fn gravity_system(
  mut player: Query<&mut FPSCamera>,
) {
  let (mut fps_camera) = player.single_mut();
  fps_camera.momentum += Vec3::new(0.0, -0.01, 0.0);
  fps_camera.momentum.y.clamp(-0.1, 999.0);
}

fn movement_input_system(
  mut player: Query<(&mut FPSCamera, &mut Transform)>,
  mut mouse_events: EventReader<MouseMotion>,
  key_events: Res<Input<KeyCode>>,
  mut windows: ResMut<Windows>,
  mut jumping: Local<bool>
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
    let mut fwd = transform.forward();
    fwd.y = 0.0;
    fwd.normalize();
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
  chunks: Query<(&Chunk)>,
  chunk_map: Res<ChunkMap>,
  time: Res<Time>
) {
  let (mut fps_camera, mut transform) = player.single_mut();
  let mut fps_camera : Mut<FPSCamera> = fps_camera;

  let mut aabbs = vec![];
  let cuboid = Cuboid::new(Vector3::new(0.5, 0.5, 0.5));
  let zero_vel = Vector3::new(0.0, 0.0, 0.0);
  let playeroid = Capsule::new(Point3::new(0.0, 0.3, 0.0), Point3::new(0.0, 1.5, 0.0), 0.3);

  for ix in -2..=2 {
    for iy in -2..=2 {
      for iz in -2..=2 {
        let c = fps_camera.pos + Vec3::new(ix as f32, iy as f32, iz as f32);
        if let Some((e, c)) = chunk_map.get_path_to_block(c) {
          if chunks.get(e).unwrap().grid[c].block != BlockId::Air {
            aabbs.push(Isometry3::translation(c.0 as f32 + 0.5, c.1 as f32 + 0.5, c.2 as f32 + 0.5));
          }
        }
      }
    }
  }

  for i in 0..3 {
    let player_pos = Isometry3::translation(fps_camera.pos.x, fps_camera.pos.y-1.5, fps_camera.pos.z);
    let player_vel = Vector3::new(fps_camera.momentum.x, fps_camera.momentum.y, fps_camera.momentum.z);
    let mut tois = vec![];

    for i in aabbs.iter() {
      match query::time_of_impact(
        &player_pos,
        &player_vel,
        &playeroid,
        &i,
        &zero_vel,
        &cuboid,
        1.0
      ) {
        Ok(f) => {
          match f {
            None => {}
            Some(TOI { toi, witness1, witness2, normal1, normal2, status }) => {
              if status == Converged {
                tois.push((toi, normal2));
              }
              if status == Penetrating {
                match query::contact(
                  &player_pos,
                  &playeroid,
                  &i,
                  &cuboid,
                  0.0
                ) {
                  Ok(Some(Contact { point1, point2, normal1, normal2, dist })) => {
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
          }
        }
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
        fps_camera.pos = fps_camera.pos + fps_camera.momentum * *toi;
        let other = Vec3::new(normal.x, normal.y, normal.z);
        fps_camera.momentum = fps_camera.momentum - fps_camera.momentum.dot(other) * other;
      }
    }
  }


  fps_camera.pos = fps_camera.pos + fps_camera.momentum;
  let y = fps_camera.momentum.y;
  fps_camera.momentum.y = 0.0;
  fps_camera.momentum *= 0.7;
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