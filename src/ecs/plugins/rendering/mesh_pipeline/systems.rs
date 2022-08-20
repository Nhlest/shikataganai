use crate::ecs::plugins::rendering::mesh_pipeline::bind_groups::{
  MeshLightBindGroup, MeshLightTextureBindGroup, MeshPositionBindGroup, MeshTextureBindGroup, MeshViewBindGroup,
};
use crate::ecs::plugins::rendering::mesh_pipeline::draw_command::DrawMeshFull;
use crate::ecs::plugins::rendering::mesh_pipeline::pipeline::MeshPipeline;
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{LightTextureHandle, TextureHandle};
use crate::ecs::resources::chunk_map::BlockAccessorReadOnly;
use crate::ecs::resources::light::LightLevel;
use crate::util::array::to_ddd;
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::extract_component::ComponentUniforms;
use bevy::render::mesh::GpuBufferInfo;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{DrawFunctions, RenderPhase};
use bevy::render::render_resource::ShaderType;
use bevy::render::render_resource::{Buffer, PipelineCache, SpecializedRenderPipelines};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::ViewUniforms;
use bevy::render::Extract;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};

#[derive(Component)]
pub struct MeshBuffer(pub Buffer, pub Buffer, pub usize);

#[derive(Component, ShaderType, Clone)]
pub struct PositionUniform {
  pub transform: Mat4,
}

pub fn extract_meshes(
  mut commands: Commands,
  meshes: Extract<Query<(&Handle<Mesh>, &Transform)>>,
  block_accessor: Extract<BlockAccessorReadOnly>,
) {
  for meshes in meshes.iter() {
    commands
      .spawn()
      .insert(PositionUniform {
        transform: Mat4::from_rotation_translation(meshes.1.rotation, meshes.1.translation),
      })
      .insert(meshes.0.clone())
      .insert(block_accessor.get_light_level(to_ddd(meshes.1.translation)).unwrap());
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
  gpu_images: Res<RenderAssets<Image>>,
  handle: Res<TextureHandle>,
  qq: Res<RenderAssets<Mesh>>,
  q: Query<(Entity, &Handle<Mesh>)>,
) {
  if let Some(gpu_image) = gpu_images.get(&handle.0) {
    commands.insert_resource(MeshTextureBindGroup {
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
        label: Some("block_material_bind_group"),
        layout: &chunk_pipeline.texture_layout,
      }),
    });
  } else {
    return;
  }

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

  for (entity, i) in q.iter() {
    if let Some(i) = qq.get(i) {
      match &i.buffer_info {
        GpuBufferInfo::Indexed { buffer, count, .. } => {
          commands
            .entity(entity)
            .insert(MeshBuffer(i.vertex_buffer.clone(), buffer.clone(), *count as usize));
          for mut view in views.iter_mut() {
            view.add(Opaque3d {
              distance: -0.2,
              draw_function,
              pipeline,
              entity,
            });
          }
        }
        GpuBufferInfo::NonIndexed { .. } => {}
      }
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
