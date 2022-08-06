use crate::ecs::components::block::Block;
use crate::ecs::plugins::camera::Selection;
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::{
  LightTextureBindGroup, LightTextureHandle, SelectionBindGroup, TextureHandle, VoxelTextureBindGroup,
  VoxelViewBindGroup,
};
use crate::ecs::plugins::rendering::voxel_pipeline::draw_command::DrawVoxelsFull;
use crate::ecs::plugins::rendering::voxel_pipeline::meshing::{ChunkMeshBuffer, RemeshEvent, SingleSide};
use crate::ecs::plugins::rendering::voxel_pipeline::pipeline::VoxelPipeline;
use crate::ecs::plugins::settings::AmbientOcclusion;
use crate::ecs::resources::chunk_map::BlockAccessorReadOnly;
use crate::util::array::{sub_ddd, ArrayIndex, ImmediateNeighbours, DD};
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{DrawFunctions, RenderPhase};
use bevy::render::render_resource::{BufferUsages, BufferVec, PipelineCache, SpecializedRenderPipelines};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::view::ViewUniforms;
use bevy::render::Extract;
use bevy::utils::hashbrown::HashMap;
use itertools::Itertools;
use wgpu::util::BufferInitDescriptor;
use wgpu::{BindGroupDescriptor, BindGroupEntry, BindingResource};

pub struct ExtractedBlocks {
  pub blocks: HashMap<DD, BufferVec<SingleSide>>,
}

impl Default for ExtractedBlocks {
  fn default() -> Self {
    Self { blocks: HashMap::new() }
  }
}

pub fn extract_chunks(
  mut commands: Commands,
  block_accessor: Extract<BlockAccessorReadOnly>,
  selection: Extract<Res<Option<Selection>>>,
  mut remesh_events: Extract<EventReader<RemeshEvent>>,
  ambient_occlusion: Extract<Res<AmbientOcclusion>>,
  mut extracted_blocks: ResMut<ExtractedBlocks>,
) {
  commands.insert_resource(selection.clone());
  let mut updated: Vec<DD> = vec![];

  for ch in remesh_events
    .iter()
    .filter_map(|p| if let RemeshEvent::Remesh(d) = p { Some(d) } else { None })
    .unique()
  {
    if !block_accessor.chunk_map.map.contains_key(ch) {
      continue;
    }
    if block_accessor.chunk_map.map[ch].entity.is_none() {
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
        for neighbour in i.immediate_neighbours() {
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
  gpu_images: Res<RenderAssets<Image>>,
  (handle, light_texture_handle): (Res<TextureHandle>, Res<LightTextureHandle>),
  selection: Res<Option<Selection>>,
  updated: Res<Vec<DD>>,
) {
  if let Some(gpu_image) = gpu_images.get(&handle.0) {
    commands.insert_resource(VoxelTextureBindGroup {
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
  }
  if let Some(gpu_image) = gpu_images.get(&light_texture_handle.0) {
    commands.insert_resource(LightTextureBindGroup {
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
        label: Some("light_texture_bind_group"),
        layout: &chunk_pipeline.light_texture_layout,
      }),
    });
  }

  if let Some(view_binding) = view_uniforms.uniforms.binding() {
    commands.insert_resource(VoxelViewBindGroup {
      bind_group: render_device.create_bind_group(&BindGroupDescriptor {
        entries: &[BindGroupEntry {
          binding: 0,
          resource: view_binding,
        }],
        label: Some("block_view_bind_group"),
        layout: &chunk_pipeline.view_layout,
      }),
    });
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

  commands.insert_resource(SelectionBindGroup {
    bind_group: render_device.create_bind_group(&BindGroupDescriptor {
      entries: &[BindGroupEntry {
        binding: 0,
        resource: BindingResource::Buffer(selection_buffer.as_entire_buffer_binding()),
      }],
      label: Some("block_view_bind_group"),
      layout: &chunk_pipeline.selection_layout,
    }),
  });

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
