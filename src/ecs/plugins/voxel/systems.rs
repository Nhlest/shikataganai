use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::ecs::system::lifetimeless::SResMut;
use bevy::prelude::*;
use bevy::render::Extract;
use bevy::render::extract_component::ComponentUniforms;
use bevy::render::mesh::GpuBufferInfo;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{DrawFunctions, RenderPhase};
use bevy::render::render_resource::{BufferUsages, BufferVec, PipelineCache, SpecializedRenderPipelines};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::ViewUniforms;
use itertools::Itertools;
use num_traits::real::Real;
use wgpu::util::BufferInitDescriptor;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};

use crate::ecs::components::block::Block;
use crate::ecs::components::chunk::Chunk;
use crate::ecs::plugins::camera::Selection;
use crate::ecs::plugins::settings::AmbientOcclusion;
use crate::ecs::plugins::voxel::{
  ChunkMeshBuffer, DrawMeshFull, DrawVoxelsFull, ExtractedBlocks, LightTextureBindGroup, LightTextureHandle,
  MeshBuffer, MeshPipeline, MeshPositionBindGroup, MeshTextureBindGroup, MeshViewBindGroup, PositionUniform,
  RemeshEvent, SelectionBindGroup, SingleSide, TextureHandle, VoxelPipeline, VoxelTextureBindGroup, VoxelViewBindGroup,
};
use crate::ecs::resources::chunk_map::{BlockAccessor, BlockAccessorReadOnly, ChunkMap};
use crate::ecs::resources::chunk_map::BlockAccessorStatic;
use crate::util::array::{sub_ddd, ArrayIndex, ImmediateNeighbours, DD};

pub fn extract_chunks(
  mut commands: Commands,
  block_accessor: Extract<BlockAccessorReadOnly>,
  selection: Extract<Res<Option<Selection>>>,
  mut remesh_events: Extract<EventReader<RemeshEvent>>,
  ambient_occlusion: Extract<Res<AmbientOcclusion>>,
  mut extracted_blocks: ResMut<ExtractedBlocks>
) {
  commands.insert_resource(selection.clone());
  let mut updated : Vec<DD> = vec![];

  for ch in remesh_events
    .iter()
    .filter_map(|p| if let RemeshEvent::Remesh(d) = p { Some(d) } else { None })
    .unique()
  {
    if !block_accessor.chunk_map.map.contains_key(ch) {
      continue;
    }
    updated.push(*ch);
    extracted_blocks
      .blocks
      .insert(*ch, BufferVec::new(BufferUsages::VERTEX));
    let extracted_blocks = extracted_blocks.blocks.get_mut(ch).unwrap();
    let entity = block_accessor.chunk_map.map[ch].entity.unwrap();
    let bounds = block_accessor.chunks.get(entity).unwrap().grid.bounds;
    let mut i = bounds.0;
    loop {
      let block: Block = block_accessor.get_single(i).unwrap().clone();
      if block.visible() {
        for neighbour in i.immeidate_neighbours() {
          if block_accessor.get_single(neighbour).map_or(true, |b| !b.visible()) {
            let light_level = block_accessor.get_light_level(neighbour);
            let lighting = match light_level {
              Some(light_level) => (light_level.heaven, light_level.hearth),
              None => (0, 0),
            };

            extracted_blocks.push(SingleSide::new(
              (i.0 as f32, i.1 as f32, i.2 as f32),
              sub_ddd(neighbour, i),
              block.block.into_array_of_faces(),
              lighting,
              &block_accessor,
              ambient_occlusion.0,
            ));
          }
        }
      }
      i = match i.next(&bounds) {
        None => break,
        Some(i) => i,
      }
    }
  }
  commands.insert_resource(updated);
}

pub fn queue_chunks(
  mut commands: Commands,
  mut extracted_blocks: ResMut<ExtractedBlocks>,
  mut views: Query<&mut RenderPhase<Opaque3d>>,
  draw_functions: Res<DrawFunctions<Opaque3d>>,
  mut pipelines: ResMut<SpecializedRenderPipelines<VoxelPipeline>>,
  mut pipeline_cache: ResMut<PipelineCache>,
  chunk_pipeline: Res<VoxelPipeline>,
  (render_device, render_queue): (Res<RenderDevice>, Res<RenderQueue>),
  view_uniforms: Res<ViewUniforms>,
  mut voxel_bind_group: ResMut<VoxelViewBindGroup>,
  mut selection_bind_group: ResMut<SelectionBindGroup>,
  gpu_images: Res<RenderAssets<Image>>,
  (handle, light_texture_handle, mut bind_group, mut light_texture_bind_group): (
    Res<TextureHandle>,
    Res<LightTextureHandle>,
    ResMut<VoxelTextureBindGroup>,
    ResMut<LightTextureBindGroup>,
  ),
  selection: Res<Option<Selection>>,
  updated: Res<Vec<DD>>,
) {
  if let Some(gpu_image) = gpu_images.get(&handle.0) {
    *bind_group = VoxelTextureBindGroup {
      bind_group: Some(render_device.create_bind_group(&BindGroupDescriptor {
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
      })),
    };
  }
  if let Some(gpu_image) = gpu_images.get(&light_texture_handle.0) {
    *light_texture_bind_group = LightTextureBindGroup {
      bind_group: Some(render_device.create_bind_group(&BindGroupDescriptor {
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
        label: Some("light_texture_bind_group"),
        layout: &chunk_pipeline.light_texture_layout,
      })),
    };
  }

  if let Some(view_binding) = view_uniforms.uniforms.binding() {
    voxel_bind_group.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
      entries: &[BindGroupEntry {
        binding: 0,
        resource: view_binding,
      }],
      label: Some("block_view_bind_group"),
      layout: &chunk_pipeline.view_layout,
    }));
  }

  let contents = match selection.into_inner() {
    None => [-9999, -9999, -9999, 0, -9999, -9999, -9999, 0],
    Some(Selection { cube, face }) => [cube.0, cube.1, cube.2, 0, face.0, face.1, face.2, 0],
  };
  let selection_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
    label: Some("selection_buffer"),
    contents: bytemuck::bytes_of(&contents),
    usage: BufferUsages::UNIFORM,
  });

  selection_bind_group.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
    entries: &[BindGroupEntry {
      binding: 0,
      resource: BindingResource::Buffer(selection_buffer.as_entire_buffer_binding()),
    }],
    label: Some("block_view_bind_group"),
    layout: &chunk_pipeline.selection_layout,
  }));

  let draw_function = draw_functions.read().get_id::<DrawVoxelsFull>().unwrap();

  let pipeline = pipelines.specialize(&mut pipeline_cache, &chunk_pipeline, ());

  let buf = &mut extracted_blocks.blocks;
  for i in updated.iter() {
    let buf = buf.get_mut(i).unwrap();
    buf.write_buffer(&render_device, &render_queue);
  }
  for (_, buf) in buf.iter_mut() {
    if !buf.is_empty() {
      let entity = commands
        .spawn()
        .insert(ChunkMeshBuffer(buf.buffer().unwrap().clone(), buf.len()))
        .id();
      for mut view in views.iter_mut() {
        view.add(Opaque3d {
          distance: 2.0,
          draw_function,
          pipeline,
          entity,
        });
      }
    }
  }
}

pub fn extract_meshes(mut commands: Commands, meshes: Extract<Query<(&Handle<Mesh>, &Transform)>>) {
  for meshes in meshes.iter() {
    commands
      .spawn()
      .insert(PositionUniform {
        transform: Mat4::from_translation(meshes.1.translation),
      })
      .insert(meshes.0.clone());
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
  mut view_bind_group: ResMut<MeshViewBindGroup>,
  gpu_images: Res<RenderAssets<Image>>,
  (handle, mut bind_group): (Res<TextureHandle>, ResMut<MeshTextureBindGroup>),
  qq: Res<RenderAssets<Mesh>>,
  q: Query<(Entity, &Handle<Mesh>)>,
) {
  if let Some(gpu_image) = gpu_images.get(&handle.0) {
    *bind_group = MeshTextureBindGroup {
      bind_group: Some(render_device.create_bind_group(&BindGroupDescriptor {
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
      })),
    };
  }
  //
  // // TODO: make it generic
  if let Some(view_binding) = view_uniforms.uniforms.binding() {
    view_bind_group.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
      entries: &[BindGroupEntry {
        binding: 0,
        resource: view_binding,
      }],
      label: Some("view_bind_group"),
      layout: &chunk_pipeline.view_layout,
    }));
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
      bind_group: Some(render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
          binding: 0,
          resource: mesh_binding.clone(),
        }],
        label: Some("mesh_position_bind_group"),
        layout: &mesh_pipeline.position_layout,
      })),
    };
    commands.insert_resource(mesh_bind_group);
  }
}
