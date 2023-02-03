use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::render_phase::{DrawFunctions, RenderPhase};
use bevy::render::render_resource::{BufferUsages, PipelineCache, SpecializedRenderPipelines};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};
use wgpu::util::BufferInitDescriptor;
use crate::ecs::plugins::game::LocalTick;
use crate::ecs::plugins::rendering::particle_pipeline::{Particle, ParticleBuffer, ParticleEmitter, ParticleVertex};
use crate::ecs::plugins::rendering::particle_pipeline::bind_groups::AspectRatioBindGroup;
use crate::ecs::plugins::rendering::particle_pipeline::draw_command::DrawParticlesFull;
use crate::ecs::plugins::rendering::particle_pipeline::pipeline::ParticlePipeline;
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{TextureBindGroup, ViewBindGroup};
use rand::prelude::*;
use shikataganai_common::ecs::resources::light::LightLevel;
use shikataganai_common::ecs::resources::world::GameWorld;

pub fn particle_system(
  mut commands: Commands,
  particle_emitters: Query<&ParticleEmitter>,
  mut particles: Query<(Entity, &mut Particle)>,
  tick: Res<LocalTick>,
  delta: Res<Time>
) {
  let mut rng = thread_rng();
  for emitter in particle_emitters.iter() {
    commands.spawn(Particle {
      location: emitter.location,
      tile: emitter.tile,
      lifetime: tick.0 + emitter.lifetime,
      velocity: Vec3::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0))
    });
  }
  for (entity, mut particle) in particles.iter_mut() {
    if particle.lifetime < tick.0 {
      commands.entity(entity).despawn();
    } else {
      let d = particle.velocity * delta.delta().as_secs_f32();
      particle.location += d;
    }
  }
}

pub fn extract_particles(
  mut commands: Commands,
  world: Extract<Res<GameWorld>>,
  particles: Extract<Query<&Particle>>
) {
  for particle in particles.iter() {
    let light = world.get_light_level((particle.location.x.floor() as i32, particle.location.y.floor() as i32, particle.location.z.floor() as i32)).unwrap_or(LightLevel::dark());
    commands.spawn(ParticleVertex {
      location: particle.location,
      tile: particle.tile as u32,
      heaven: light.heaven as u16,
      hearth: light.hearth as u16,
    });
  }
}

#[derive(Resource)]
pub struct Ratio(pub f32);

pub fn extract_aspect_ratio(
  mut commands: Commands,
  window: Extract<Res<Windows>>,
) {
  let ratio = window.primary().width() / window.primary().height();
  commands.insert_resource(Ratio(ratio));
}

pub fn queue_particles(
  mut commands: Commands,
  mut particle_buf: ResMut<ParticleBuffer>,
  particles: Query<&ParticleVertex>,
  device: Res<RenderDevice>,
  queue: Res<RenderQueue>,
  ratio: Res<Ratio>,

  particle_pipeline: Res<ParticlePipeline>,
  mut views: Query<&mut RenderPhase<Opaque3d>>,
  draw_functions: Res<DrawFunctions<Opaque3d>>,
  mut pipelines: ResMut<SpecializedRenderPipelines<ParticlePipeline>>,
  mut pipeline_cache: ResMut<PipelineCache>,

  view_bind_group: Option<Res<ViewBindGroup>>,
  texture_bind_group: Option<Res<TextureBindGroup>>,
) {
  particle_buf.count = 0;
  particle_buf.particles.clear();
  for particle in particles.iter() {
    particle_buf.particles.push(particle.clone());
    particle_buf.count+=1;
  }
  particle_buf.particles.write_buffer(device.as_ref(), queue.as_ref());

  if view_bind_group.is_none() || texture_bind_group.is_none() {
    return;
  }

  let buffer = device.create_buffer_with_data(&BufferInitDescriptor {
    label: None,
    contents: &ratio.0.to_le_bytes(),
    usage: BufferUsages::UNIFORM,
  });

  commands.insert_resource(AspectRatioBindGroup {
    bind_group: device.create_bind_group(&BindGroupDescriptor {
      entries: &[
        BindGroupEntry {
          binding: 0,
          resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
        },
      ],
      label: Some("aspect_ratio_bind_group"),
      layout: &particle_pipeline.aspect_ratio_layout,
    }),
  });

  let draw_function = draw_functions.read().get_id::<DrawParticlesFull>().unwrap();

  let pipeline = pipelines.specialize(&mut pipeline_cache, &particle_pipeline, ());

  let entity = commands.spawn_empty().id();
  for mut view in views.iter_mut() {
    view.add(Opaque3d {
      distance: 0.1,
      draw_function,
      pipeline,
      entity,
    });
  }
}