use crate::ecs::plugins::rendering::mesh_pipeline::bind_groups::{
  MeshLightBindGroup, MeshLightTextureBindGroup, MeshPositionBindGroup, MeshViewBindGroup,
};
use crate::ecs::plugins::rendering::mesh_pipeline::draw_command::DrawMeshFull;
use crate::ecs::plugins::rendering::mesh_pipeline::pipeline::{MeshPipeline, RenderTextures};
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{LightTextureHandle, TextureHandle};
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::extract_component::ComponentUniforms;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{DrawFunctions, RenderPhase};
use bevy::render::render_resource::{Buffer, PipelineCache, SpecializedRenderPipelines};
use bevy::render::render_resource::{IndexFormat, ShaderType};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::ViewUniforms;
use bevy::render::Extract;
use shikataganai_common::ecs::resources::light::LightLevel;
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::util::array::to_ddd;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};

#[derive(Component)]
pub struct MeshBuffer(pub Buffer, pub Buffer, pub usize, pub IndexFormat);

#[derive(Component)]
pub struct MeshMarker;

#[derive(Component, ShaderType, Clone)]
pub struct PositionUniform {
  pub transform: Mat4,
}

pub fn extract_meshes(
  mut commands: Commands,
  meshes: Extract<Query<(&Handle<Mesh>, &GlobalTransform, Option<&Handle<Image>>), With<MeshMarker>>>,
  game_world: Extract<Res<GameWorld>>,
  default_texture: Res<TextureHandle>,
) {
  for (mesh, transform, image) in meshes.iter() {
    if let Some(light_level) = game_world.get_light_level(to_ddd(transform.translation())) {
      commands.spawn((
        PositionUniform {
          // transform: Mat4::from_rotation_translation(transform, transform.translation),
          transform: transform.compute_matrix(),
        },
        MeshMarker,
        mesh.clone(),
        light_level,
        image.cloned().unwrap_or(default_texture.0.clone()),
      ));
    }
  }
}

pub fn prepare_textures(
  gpu_images: Res<RenderAssets<Image>>,
  render_device: Res<RenderDevice>,
  mesh_pipeline: Res<MeshPipeline>,
  mut render_textures: ResMut<RenderTextures>,
  query: Query<&Handle<Image>>,
) {
  for handle in query.iter() {
    if let Some(gpu_image) = gpu_images.get(handle) {
      let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[
          BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&gpu_image.texture_view),
          },
          BindGroupEntry {
            binding: 1,
            resource: BindingResource::Sampler(&gpu_image.sampler),
          },
        ],
        label: Some("block_material_bind_group"),
        layout: &mesh_pipeline.texture_layout,
      });
      render_textures.insert(handle.clone(), bind_group);
    }
  }
}

pub fn queue_meshes(
  mut commands: Commands,
  mut views: Query<&mut RenderPhase<Opaque3d>>,
  draw_functions: Res<DrawFunctions<Opaque3d>>,
  mut pipelines: ResMut<SpecializedRenderPipelines<MeshPipeline>>,
  mut pipeline_cache: ResMut<PipelineCache>,
  chunk_pipeline: Res<MeshPipeline>,
  render_device: Res<RenderDevice>,
  view_uniforms: Res<ViewUniforms>,
  q: Query<Entity, With<MeshMarker>>,
) {
  if let Some(view_binding) = view_uniforms.uniforms.binding() {
    commands.insert_resource(MeshViewBindGroup {
      bind_group: render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
          binding: 0,
          resource: view_binding,
        }],
        label: Some("view_bind_group"),
        layout: &chunk_pipeline.view_layout,
      }),
    });
  } else {
    return;
  }

  let draw_function = draw_functions.read().get_id::<DrawMeshFull>().unwrap();
  let pipeline = pipelines.specialize(&mut pipeline_cache, &chunk_pipeline, ());

  for entity in q.iter() {
    for mut view in views.iter_mut() {
      view.add(Opaque3d {
        distance: -0.2,
        draw_function,
        pipeline,
        entity,
      });
    }
  }
}

pub fn queue_mesh_position_bind_group(
  mut commands: Commands,
  mesh_pipeline: Res<MeshPipeline>,
  render_device: Res<RenderDevice>,
  mesh_uniforms: Res<ComponentUniforms<PositionUniform>>,
) {
  if let Some(mesh_binding) = mesh_uniforms.uniforms().binding() {
    let mesh_bind_group = MeshPositionBindGroup {
      bind_group: render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
          binding: 0,
          resource: mesh_binding.clone(),
        }],
        label: Some("mesh_position_bind_group"),
        layout: &mesh_pipeline.position_layout,
      }),
    };
    commands.insert_resource(mesh_bind_group);
  }
}

pub fn queue_light_levels_bind_group(
  mut commands: Commands,
  mesh_pipeline: Res<MeshPipeline>,
  render_device: Res<RenderDevice>,
  light_level_uniforms: Res<ComponentUniforms<LightLevel>>,
) {
  if let Some(light_binding) = light_level_uniforms.uniforms().binding() {
    let light_bind_group = MeshLightBindGroup {
      bind_group: render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
          binding: 0,
          resource: light_binding.clone(),
        }],
        label: Some("mesh_light_bind_group"),
        layout: &mesh_pipeline.light_layout,
      }),
    };
    commands.insert_resource(light_bind_group);
  }
}

pub fn queue_light_texture_bind_group(
  mut commands: Commands,
  mesh_pipeline: Res<MeshPipeline>,
  render_device: Res<RenderDevice>,
  gpu_images: Res<RenderAssets<Image>>,
  light_texture_handle: Res<LightTextureHandle>,
) {
  if let Some(gpu_image) = gpu_images.get(&light_texture_handle.0) {
    commands.insert_resource(MeshLightTextureBindGroup {
      bind_group: render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[
          BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&gpu_image.texture_view),
          },
          BindGroupEntry {
            binding: 1,
            resource: BindingResource::Sampler(&gpu_image.sampler),
          },
        ],
        label: Some("mesh_light_texture_bind_group"),
        layout: &mesh_pipeline.light_texture_layout,
      }),
    });
  }
}
