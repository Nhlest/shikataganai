use crate::ecs::components::block::BlockId;
use crate::ecs::components::chunk::Chunk;
use crate::ecs::plugins::settings::MouseSensitivity;
use crate::ecs::resources::chunk_map::ChunkMap;
use crate::util::array::{to_ddd, DDD};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::camera::CameraProjection;
use bevy::render::primitives::Frustum;
use bevy::tasks::AsyncComputeTaskPool;
use bevy_rapier3d::parry::query::Ray;
use bevy_rapier3d::prelude::*;
use num_traits::float::FloatConst;

pub struct CameraPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FPSCamera {
  phi: f32,
  theta: f32,
  phi_a: f32,
  theta_a: f32,
}

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    let fps_camera = FPSCamera {
      phi: 0.0,
      theta: f32::FRAC_PI_2(),
      phi_a: 0.0,
      theta_a: 0.0,
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
      .insert(RigidBody::Dynamic)
      .insert(Transform::from_xyz(10.1, 20.0, 10.0))
      .insert(GlobalTransform::default())
      .insert(LockedAxes::ROTATION_LOCKED)
      .insert(Player)
      .insert(Velocity::default())
      .insert(GravityScale(2.0))
      .insert(Friction {
        coefficient: 0.0,
        combine_rule: CoefficientCombineRule::Min,
      })
      .with_children(|c| {
        c.spawn()
          .insert(GlobalTransform::default())
          .insert(Transform::from_xyz(0.0, -0.5, 0.0))
          .insert(Collider::capsule_y(0.6, 0.3))
          .insert(SolverGroups::new(0b01, 0b10))
          .insert(CollisionGroups::new(0b01, 0b10));
        c.spawn().insert(fps_camera).insert_bundle(camera);
      });
    app
      .add_system(movement_input_system)
      .add_system_to_stage(CoreStage::PreUpdate, collision_movement_system)
      .add_system_to_stage(CoreStage::Update, block_pick)
      .add_system(cursor_grab_system);
  }
}

fn movement_input_system(
  mut player: Query<&mut FPSCamera>,
  mut camera: Query<&mut Velocity, With<Player>>,
  camera_transform: Query<&Transform, With<Camera>>,
  mut mouse_events: EventReader<MouseMotion>,
  key_events: Res<Input<KeyCode>>,
  mut windows: ResMut<Windows>,
  mut stationary_frames: Local<i32>,
  mouse_sensitivity: Res<MouseSensitivity>,
  time: Res<Time>,
) {
  let window = windows.get_primary_mut().unwrap();
  if !window.cursor_locked() {
    return;
  }

  let mut fps_camera = player.single_mut();
  let mut camera_velocity = camera.single_mut();
  let transform = camera_transform.single();

  for MouseMotion { delta } in mouse_events.iter() {
    fps_camera.phi_a += delta.x * mouse_sensitivity.0 * 0.04;
    fps_camera.theta_a = fps_camera.theta_a + delta.y * mouse_sensitivity.0 * 0.04;
  }

  fps_camera.phi += fps_camera.phi_a * time.delta().as_secs_f32();
  fps_camera.theta = (fps_camera.theta + fps_camera.theta_a * time.delta().as_secs_f32()).clamp(0.05, f32::PI() - 0.05);

  fps_camera.phi_a *= 0.70;
  fps_camera.theta_a *= 0.70;

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

  if camera_velocity.linvel.y.abs() < 0.001 {
    *stationary_frames = *stationary_frames + 1;
  } else {
    *stationary_frames = 0; // TODO: potential for a double jump here;
  }

  if key_events.pressed(KeyCode::Space) && *stationary_frames > 2 {
    *stationary_frames = 0;
    camera_velocity.linvel.y = 7.0;
  }

  let y = camera_velocity.linvel.y;
  camera_velocity.linvel.y = 0.0;
  camera_velocity.linvel = movement * 5.0;
  camera_velocity.linvel.y = y;
}

#[derive(Component)]
pub struct Cube;

fn collision_movement_system(
  camera: Query<(Entity, &FPSCamera)>,
  player: Query<Entity, With<Player>>,
  mut queries: ParamSet<(Query<&mut Transform>, Query<&mut Transform, With<Cube>>)>,
  mut commands: Commands,
  chunks: Query<&Chunk>,
  mut chunk_map: ResMut<ChunkMap>,
  dispatcher: Res<AsyncComputeTaskPool>,
) {
  let (entity_camera, fps_camera) = camera.single();
  let entity_player = player.single();
  let translation = {
    let q = queries.p0();
    q.get(entity_player).unwrap().translation
  };

  let mut query = queries.p1();
  let mut iter = query.iter_mut();

  for ix in -3..=3 {
    for iy in -3..=3 {
      for iz in -3..=3 {
        let c = translation + Vec3::new(ix as f32, iy as f32, iz as f32);
        let c = to_ddd(c);
        if let Some(block) = chunk_map.get(&mut commands, &dispatcher, &chunks, c) {
          if block.block != BlockId::Air {
            match iter.next() {
              None => {
                commands
                  .spawn()
                  .insert(RigidBody::Fixed)
                  .insert(Collider::cuboid(0.5, 0.5, 0.5))
                  .insert(Cube)
                  .insert(Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                  })
                  .insert(SolverGroups::new(0b10, 0b01))
                  .insert(CollisionGroups::new(0b10, 0b01))
                  .insert(Transform::from_xyz(
                    c.0 as f32 + 0.5,
                    c.1 as f32 + 0.5,
                    c.2 as f32 + 0.5,
                  ))
                  .insert(GlobalTransform::default());
              }
              Some(mut transform) => {
                transform.translation = Vec3::new(c.0 as f32 + 0.5, c.1 as f32 + 0.5, c.2 as f32 + 0.5);
              }
            }
          }
        }
      }
    }
  }
  for mut transform in iter {
    transform.translation = Vec3::new(-9999.0, -9999.0, -9999.0);
  }
  drop(query);
  let mut transforms = queries.p0();

  let looking_at = Vec3::new(
    10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
    10.0 * fps_camera.theta.cos(),
    10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
  );

  let mut camera_t = transforms.get_mut(entity_camera).unwrap();
  camera_t.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));
}

pub struct MainMenuOpened(pub bool);

fn cursor_grab_system(
  mut windows: ResMut<Windows>,
  key: Res<Input<KeyCode>>,
  mut main_menu_opened: ResMut<MainMenuOpened>,
) {
  let window = windows.get_primary_mut().unwrap();

  if key.just_pressed(KeyCode::Escape) {
    if main_menu_opened.0 {
      window.set_cursor_lock_mode(true);
      window.set_cursor_visibility(false);
    } else {
      window.set_cursor_lock_mode(false);
      window.set_cursor_visibility(true);
    }
    main_menu_opened.0 = !main_menu_opened.0;
  }
}

#[derive(Clone, Debug)]
pub struct Selection {
  pub cube: DDD,
  pub face: DDD,
}

fn block_pick(
  camera: Query<&GlobalTransform, With<Camera>>,
  transforms: Query<&Transform>,
  mut selection: ResMut<Option<Selection>>,
  rapier_context: Res<RapierContext>,
) {
  *selection = None;
  let transform = camera.single();
  let origin = transform.translation;
  let direction = transform.forward();

  if let Some((entity, intersection)) = rapier_context.query_pipeline.cast_ray_and_get_normal(
    &rapier_context.colliders,
    &Ray::new(origin.into(), direction.into()),
    5.0,
    false,
    InteractionGroups::new(0b01, 0b10),
    None,
  ) {
    let c = rapier_context.colliders.get(entity).unwrap();
    let e = Entity::from_bits(c.user_data as u64);
    let transform = transforms.get(e).unwrap();
    let cube = transform.translation - Vec3::new(0.5, 0.5, 0.5);
    let normal: Vec3 = Vec3::from(intersection.normal) + cube;
    *selection = Some(Selection {
      cube: (cube.x.round() as i32, cube.y.floor() as i32, cube.z.floor() as i32),
      face: (
        normal.x.floor() as i32,
        normal.y.floor() as i32,
        normal.z.floor() as i32,
      ),
    });
  }
}
