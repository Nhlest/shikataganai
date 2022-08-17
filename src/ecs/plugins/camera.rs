// TODO: collision system is still too lidl, fixme
use crate::ecs::components::blocks::BlockRenderInfo;
use crate::ecs::plugins::game::{in_game, ShikataganaiGameState};
use crate::ecs::plugins::rendering::mesh_pipeline::loader::GltfMeshStorageHandle;
use crate::ecs::plugins::settings::MouseSensitivity;
use crate::ecs::resources::chunk_map::{BlockAccessor, BlockAccessorInternal, BlockAccessorSpawner};
use crate::util::array::{to_ddd, DDD};
use crate::GltfMeshStorage;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, Projection};
use bevy::render::primitives::Frustum;
use bevy_rapier3d::prelude::TOIStatus::Converged;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::{ConditionSet, CurrentState, IntoConditionalSystem};
use iyes_loopless::state::NextState;
use num_traits::float::FloatConst;

pub struct CameraPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCollider;

#[derive(Component)]
pub struct FPSCamera {
  pub phi: f32,
  pub theta: f32,
  pub velocity: Vect,
}

#[derive(Default)]
pub struct PlayerPreviousPosition(pub DDD);

impl Plugin for CameraPlugin {
  fn build(&self, app: &mut App) {
    let fps_camera = FPSCamera {
      phi: 0.0,
      theta: f32::FRAC_PI_2(),
      velocity: Vect::ZERO,
    };
    let camera = {
      let perspective_projection = PerspectiveProjection {
        fov: std::f32::consts::PI / 1.8,
        near: 0.001,
        far: 1000.0,
        aspect_ratio: 1.0,
      };
      let view_projection = perspective_projection.get_projection_matrix();
      let frustum =
        Frustum::from_view_projection(&view_projection, &Vec3::ZERO, &Vec3::Z, perspective_projection.far());
      Camera3dBundle {
        projection: Projection::Perspective(perspective_projection),
        frustum,
        ..default()
      }
    };
    app
      .world
      .spawn()
      .insert(RigidBody::KinematicPositionBased)
      .insert(Transform::from_xyz(10.1, 45.0, 10.0))
      .insert(GlobalTransform::default())
      .insert(LockedAxes::ROTATION_LOCKED)
      .insert(Player)
      .insert(Velocity::default())
      .with_children(|c| {
        c.spawn()
          .insert(PlayerCollider)
          .insert(GlobalTransform::default())
          .insert(Transform::from_xyz(0.0, -0.5, 0.0))
          .insert(Collider::cylinder(0.8, 0.2))
          .insert(SolverGroups::new(0b01, 0b10))
          .insert(CollisionGroups::new(0b01, 0b10));
        c.spawn().insert(fps_camera).insert_bundle(camera);
      });

    let on_game_simulation_continous = ConditionSet::new()
      .run_in_state(ShikataganaiGameState::Simulation)
      .with_system(movement_input_system)
      .with_system(collision_movement_system)
      .into();
    app.add_system_set(on_game_simulation_continous);
    app.add_system(cursor_grab_system.run_if(in_game));
    let on_simulation_pre_update = ConditionSet::new()
      .run_in_state(ShikataganaiGameState::Simulation)
      .with_system(block_pick)
      .into();
    app.add_system_set_to_stage(CoreStage::PreUpdate, on_simulation_pre_update);
    app.init_resource::<PlayerPreviousPosition>();
  }
}

fn movement_input_system(
  mut player: Query<&mut FPSCamera>,
  camera_transform: Query<&Transform, With<Camera>>,
  mut mouse_events: EventReader<MouseMotion>,
  key_events: Res<Input<KeyCode>>,
  mut windows: ResMut<Windows>,
  mut stationary_frames: Local<i32>,
  mouse_sensitivity: Res<MouseSensitivity>,
  time: Res<Time>,
  player_position: Query<&Transform, With<Player>>,
  mut block_accessor: BlockAccessorSpawner,
) {
  let translation = player_position.single().translation;

  if block_accessor.get_chunk_entity_or_queue(to_ddd(translation)).is_none() {
    return;
  }

  let window = windows.get_primary_mut().unwrap();
  let mut movement = Vec3::default();
  let mut fps_camera = player.single_mut();
  let transform = camera_transform.single();

  if window.cursor_locked() {
    for MouseMotion { delta } in mouse_events.iter() {
      fps_camera.phi += delta.x * mouse_sensitivity.0 * 0.003;
      fps_camera.theta = (fps_camera.theta + delta.y * mouse_sensitivity.0 * 0.003).clamp(0.00005, f32::PI() - 0.00005);
    }

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

    if key_events.pressed(KeyCode::Space) && *stationary_frames > 2 {
      *stationary_frames = 0;
      fps_camera.velocity.y = 7.0;
    }
  }

  movement = movement.normalize_or_zero();

  if fps_camera.velocity.y.abs() < 0.001 {
    *stationary_frames = *stationary_frames + 1;
  } else {
    *stationary_frames = 0; // TODO: potential for a double jump here;
  }

  let y = fps_camera.velocity.y;
  fps_camera.velocity.y = 0.0;
  fps_camera.velocity = movement.into();
  fps_camera.velocity *= 5.0;
  fps_camera.velocity.y = y;
  fps_camera.velocity.y -= 19.8 * time.delta().as_secs_f32().clamp(0.0, 0.1);
}

#[derive(Component)]
pub struct Cube;

//TODO: too lidl, fix
fn collision_movement_system(
  mut commands: Commands,
  mut camera: Query<(Entity, &mut FPSCamera)>,
  player: Query<Entity, With<Player>>,
  mut queries: ParamSet<(Query<&mut Transform>, Query<(Entity, &mut Transform), With<Cube>>)>,
  mut block_accessor: BlockAccessorSpawner,
  time: Res<Time>,
  rapier_context: Res<RapierContext>,
  mesh_assets: Res<Assets<Mesh>>,
  storage: Res<GltfMeshStorageHandle>,
  mesh_storage_assets: Res<Assets<GltfMeshStorage>>,
  mut player_previous_position: ResMut<PlayerPreviousPosition>
) {
  let (entity_camera, mut fps_camera): (Entity, Mut<FPSCamera>) = camera.single_mut();
  let entity_player = player.single();
  let translation = {
    let q = queries.p0();
    q.get(entity_player).unwrap().translation
  };

  //TODO: doesnt work. At all
  if block_accessor.get_chunk_entity_or_queue(to_ddd(translation)).is_none() {
    return;
  }

  let mut query = queries.p1();


  let player_new_position = to_ddd(translation);
  if player_new_position != player_previous_position.0 {
    let iter = query.iter_mut();
    for (e, _) in iter {
      commands.entity(e).despawn();
    }
    for ix in -3..=3 {
      for iy in -3..=3 {
        for iz in -3..=3 {
          let c = translation + Vec3::new(ix as f32, iy as f32, iz as f32);
          let c = to_ddd(c);
          if let Some(block) = block_accessor.get_single(c) {
            if !block.passable() {
              match block.render_info() {
                BlockRenderInfo::AsBlock(_) => {
                  block_accessor
                    .commands
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
                BlockRenderInfo::AsMesh(mesh) => {
                  if let Some(mesh_assets_hash_map) = mesh_storage_assets.get(&storage.0) {
                    let mesh = &mesh_assets_hash_map[&mesh];
                    let collider_mesh = mesh_assets.get(&mesh.collision.as_ref().unwrap()).unwrap();
                    let meta = block.meta.v as f32;
                    block_accessor
                      .commands
                      .spawn()
                      .insert(RigidBody::Fixed)
                      .insert(Collider::from_bevy_mesh(collider_mesh, &ComputedColliderShape::TriMesh).unwrap())
                      .insert(Cube)
                      .insert(Friction {
                        coefficient: 0.0,
                        combine_rule: CoefficientCombineRule::Min,
                      })
                      .insert(SolverGroups::new(0b10, 0b01))
                      .insert(CollisionGroups::new(0b10, 0b01))
                      .insert(
                        Transform::from_xyz(c.0 as f32 + 0.5, c.1 as f32 + 0.5, c.2 as f32 + 0.5)
                          .with_rotation(Quat::from_rotation_y(f32::FRAC_PI_2() * meta)),
                      )
                      .insert(GlobalTransform::default());
                  }
                }
                _ => {}
              }
            }
          }
        }
      }
    }
    drop(query);
    player_previous_position.0 = player_new_position;
  }
  let mut transforms = queries.p0();

  let looking_at = Vec3::new(
    10.0 * fps_camera.phi.cos() * fps_camera.theta.sin(),
    10.0 * fps_camera.theta.cos(),
    10.0 * fps_camera.phi.sin() * fps_camera.theta.sin(),
  );

  let mut camera_t = transforms.get_mut(entity_camera).unwrap();
  camera_t.look_at(looking_at, Vec3::new(0.0, 1.0, 0.0));

  let shape = Collider::cylinder(0.745, 0.2);
  let feet_shape = Collider::cylinder(0.05, 0.2);

  let mut movement_left = fps_camera.velocity * time.delta().as_secs_f32();
  let leg_height = 0.26;

  let filter = QueryFilter {
    flags: Default::default(),
    groups: Some(InteractionGroups::new(0b01, 0b10)),
    exclude_collider: None,
    exclude_rigid_body: None,
    predicate: None,
  };

  loop {
    if movement_left.length() <= 0.0 {
      break;
    }
    let mut position = transforms.get(entity_player).unwrap().translation - Vec3::new(0.0, 0.495, 0.0);

    let intersection = rapier_context.cast_shape(position, Rot::default(), movement_left, &shape, 1.0, filter);

    match intersection {
      None => {
        position = position + movement_left + Vec3::new(0.0, 0.495, 0.0);
        transforms.get_mut(entity_player).unwrap().translation = position;
        break;
      }
      Some((collision_entity, toi)) => {
        if toi.status != Converged {
          //TODO: fix this, this is so lidl i cant
          let unstuck_vector = transforms.get(collision_entity).unwrap().translation - position;
          transforms.get_mut(entity_player).unwrap().translation -= unstuck_vector.normalize() * 0.01;
          fps_camera.velocity = Vec3::new(0.0, 0.0, 0.0);
          break;
        }
        let v = movement_left.dot(toi.normal1);
        movement_left = movement_left - v * toi.normal1;
        fps_camera.velocity = movement_left / time.delta().as_secs_f32();
      }
    }
  }

  // ----------------------------  TODO:   -----------------------------------------
  // While mid air treat entire player body as one tall collider
  // If just went from on_the_ground to mid_air have a snap leg raycast to snap down
  // While on the ground do that leg thing
  // -------------------------------------------------------------------------------

  if fps_camera.velocity.y <= 0.0 {
    let position = transforms.get(entity_player).unwrap().translation - Vec3::new(0.0, 1.19, 0.0);

    let intersection = rapier_context.cast_shape(
      position,
      Rot::default(),
      Vec3::new(0.0, -1.0, 0.0),
      &feet_shape,
      leg_height,
      filter,
    );

    match intersection {
      None => {}
      Some((_, toi)) => {
        transforms.get_mut(entity_player).unwrap().translation -= Vec3::new(0.0, toi.toi - leg_height, 0.0);
        fps_camera.velocity.y = 0.0;
      }
    }

    // TODO: downward snapping
    // if fps_camera.velocity.y == 0.0 {
    //   let position = transforms.get(entity_player).unwrap().translation - Vec3::new(0.0, 1.45, 0.0);
    //
    //   let intersection = rapier_context.cast_shape(
    //     position,
    //     Rot::default(),
    //     Vec3::new(0.0, -1.0, 0.0),
    //     &feet_shape,
    //     leg_height+0.5,
    //     filter
    //   );
    //
    //   match intersection {
    //     None => {}
    //     Some((_, toi)) => {
    //       transforms.get_mut(entity_player).unwrap().translation -= Vec3::new(0.0, toi.toi - leg_height, 0.0);
    //       fps_camera.velocity.y = 0.0;
    //     }
    //   }
    // }
  }
}

fn cursor_grab_system(
  mut commands: Commands,
  current_state: Res<CurrentState<ShikataganaiGameState>>,
  mut windows: ResMut<Windows>,
  key: Res<Input<KeyCode>>,
  mut physics_system: ResMut<RapierConfiguration>,
) {
  let window = windows.get_primary_mut().unwrap();

  if key.just_pressed(KeyCode::Escape) {
    match current_state.0 {
      ShikataganaiGameState::Simulation => {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
        commands.insert_resource(NextState(ShikataganaiGameState::Paused));
        physics_system.physics_pipeline_active = false;
      }
      ShikataganaiGameState::Paused => {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
        commands.insert_resource(NextState(ShikataganaiGameState::Simulation));
        physics_system.physics_pipeline_active = true;
      }
      _ => {}
    }
  }
}

#[derive(Clone, Debug, Default)]
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
  let origin = transform.translation();
  let direction = transform.forward();

  if let Some((entity, intersection)) = rapier_context.cast_ray_and_get_normal(
    origin.into(),
    direction.into(),
    5.0,
    false,
    QueryFilter {
      flags: Default::default(),
      groups: Some(InteractionGroups::new(0b01, 0b10)),
      exclude_collider: None,
      exclude_rigid_body: None,
      predicate: None,
    },
  ) {
    // let c = rapier_context.colliders.get(entity).unwrap();
    // let e = Entity::from_bits(c.user_data as u64);
    let transform = transforms.get(entity).unwrap();
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
