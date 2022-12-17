use crate::ecs::plugins::rendering::skybox_pipeline::bind_groups::{
  SkyboxMeshPositionBindGroup, SkyboxTextureBindGroup, SkyboxViewBindGroup,
};
use crate::ecs::plugins::rendering::skybox_pipeline::draw_command::DrawSkyboxFull;
use crate::ecs::plugins::rendering::skybox_pipeline::pipeline::SkyboxPipeline;
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::pbr::MeshUniform;
use bevy::prelude::*;
use bevy::render::extract_component::ComponentUniforms;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{DrawFunctions, RenderPhase};
use bevy::render::render_resource::{PipelineCache, SpecializedRenderPipelines};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::ViewUniforms;
use bevy::render::Extract;
use bevy_atmosphere::pipeline::AtmosphereImage;
use bevy_atmosphere::skybox::{AtmosphereSkyBoxMaterial, SkyBoxMaterial};
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};

#[derive(Resource)]
pub struct ExtractedAtmosphereSkyBoxMaterial(pub Handle<SkyBoxMaterial>);

pub fn extract_skybox_material_handle(
  mut commands: Commands,
  skybox_material_handle: Extract<Res<AtmosphereSkyBoxMaterial>>,
  skybox_material: Extract<Query<(&Handle<Mesh>, &GlobalTransform), With<Handle<SkyBoxMaterial>>>>,
) {
  commands.insert_resource(ExtractedAtmosphereSkyBoxMaterial(skybox_material_handle.0.clone()));
  for (handle, transform) in skybox_material.iter() {
    commands.spawn((
      MeshUniform {
        transform: transform.compute_matrix(),
        inverse_transpose_model: transform.compute_matrix().inverse().transpose(),
        flags: 0,
      },
      handle.clone(),
    ));
  }
}

pub fn queue_skybox_mesh_position_bind_group(
  mut commands: Commands,
  skybox_pipeline: Res<SkyboxPipeline>,
  render_device: Res<RenderDevice>,
  mesh_uniforms: Res<ComponentUniforms<MeshUniform>>,
) {
  if let Some(mesh_binding) = mesh_uniforms.uniforms().binding() {
    let mesh_bind_group = SkyboxMeshPositionBindGroup {
      bind_group: render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
          binding: 0,
          resource: mesh_binding.clone(),
        }],
        label: Some("skybox_mesh_position_bind_group"),
        layout: &skybox_pipeline.mesh_layout,
      }),
    };
    commands.insert_resource(mesh_bind_group);
  }
}

pub fn queue_skybox(
  mut commands: Commands,
  mut views: Query<&mut RenderPhase<Opaque3d>>,
  draw_functions: Res<DrawFunctions<Opaque3d>>,
  mut pipelines: ResMut<SpecializedRenderPipelines<SkyboxPipeline>>,
  mut pipeline_cache: ResMut<PipelineCache>,
  skybox_pipeline: Res<SkyboxPipeline>,
  render_device: Res<RenderDevice>,
  view_uniforms: Res<ViewUniforms>,
  skybox_meshes: Query<Entity, With<MeshUniform>>,
  skybox_texture: Res<AtmosphereImage>,
  gpu_images: Res<RenderAssets<Image>>,
) {
  if let Some(gpu_image) = gpu_images.get(&skybox_texture.handle) {
    commands.insert_resource(SkyboxTextureBindGroup {
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
        label: Some("skybox_texture_bind_group"),
        layout: &skybox_pipeline.texture_layout,
      }),
    });
  }
  if let Some(view_binding) = view_uniforms.uniforms.binding() {
    commands.insert_resource(SkyboxViewBindGroup {
      bind_group: render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
          binding: 0,
          resource: view_binding,
        }],
        label: Some("view_bind_group"),
        layout: &skybox_pipeline.view_layout,
      }),
    });
  } else {
    return;
  }

  let draw_function = draw_functions.read().get_id::<DrawSkyboxFull>().unwrap();
  let pipeline = pipelines.specialize(&mut pipeline_cache, &skybox_pipeline, ());

  for entity in &skybox_meshes {
    for mut view in views.iter_mut() {
      view.add(Opaque3d {
        distance: 100.0,
        draw_function,
        pipeline,
        entity,
      });
    }
  }
}
