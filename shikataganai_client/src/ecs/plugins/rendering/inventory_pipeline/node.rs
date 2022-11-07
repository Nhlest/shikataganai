use crate::ecs::components::blocks::{BlockRenderInfo, DerefExt};
use crate::ecs::plugins::rendering::inventory_pipeline::inventory_cache::{ItemRenderEntry, ItemRenderMap};
use crate::ecs::plugins::rendering::inventory_pipeline::pipeline::InventoryNode;
use crate::ecs::plugins::rendering::inventory_pipeline::{INVENTORY_OUTPUT_TEXTURE_WIDTH, TEXTURE_NODE_OUTPUT_SLOT};
use crate::ecs::plugins::rendering::mesh_pipeline::loader::{GltfMeshStorage, GltfMeshStorageHandle};
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::TextureHandle;
use crate::ecs::resources::block::BlockSprite;
use bevy::prelude::*;
use bevy::render::mesh::GpuBufferInfo;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType, SlotValue};
use bevy::render::render_resource::{BufferUsages, PipelineCache};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bytemuck_derive::*;
use image::EncodableLayout;
use num_traits::FloatConst;
use shikataganai_common::ecs::components::blocks::BlockOrItem;
use std::collections::HashMap;
use wgpu::util::BufferInitDescriptor;
use wgpu::IndexFormat;

const HEX_CC: [f32; 2] = [0.000000, -0.000000];
const HEX_NN: [f32; 2] = [0.000000, -1.000000];
const HEX_SS: [f32; 2] = [0.000000, 1.000000];
const HEX_NW: [f32; 2] = [-0.866025, -0.500000];
const HEX_NE: [f32; 2] = [0.866025, -0.500000];
const HEX_SW: [f32; 2] = [-0.866025, 0.500000];
const HEX_SE: [f32; 2] = [0.866025, 0.500000];

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
pub struct MatrixContent {
  mat: Mat4,
  filler1: [u8; 64],
  filler2: [u8; 64],
  filler3: [u8; 64],
}

pub struct Initialised;

impl Node for InventoryNode {
  fn input(&self) -> Vec<SlotInfo> {
    Vec::new()
  }
  fn output(&self) -> Vec<SlotInfo> {
    vec![SlotInfo {
      name: TEXTURE_NODE_OUTPUT_SLOT.into(),
      slot_type: SlotType::TextureView,
    }]
  }
  fn update(&mut self, world: &mut World) {
    if !world.contains_resource::<ItemRenderMap>() {
      return;
    }

    self.direct_render_pipeline.render_pipeline = world
      .resource_mut::<PipelineCache>()
      .get_render_pipeline(self.direct_render_pipeline.pipeline_id)
      .cloned();

    self.mesh_render_pipeline.render_pipeline = world
      .resource_mut::<PipelineCache>()
      .get_render_pipeline(self.mesh_render_pipeline.pipeline_id)
      .cloned();

    let handle = world.resource::<TextureHandle>();
    let gpu_images = world.resource::<RenderAssets<Image>>();
    self.initialised = self.mesh_render_pipeline.render_pipeline.is_some()
      && self.direct_render_pipeline.render_pipeline.is_some()
      && gpu_images.get(&handle.0).is_some();

    if !self.initialised {
      self.to_render = true;
      self.view = InventoryNode::create_view(world.resource::<RenderDevice>());
      self.depth_view = InventoryNode::create_depth_view(world.resource::<RenderDevice>());
    } else {
      world.insert_resource(Initialised);
      self.to_render = false;
    }
  }
  fn run(
    &self,
    graph: &mut RenderGraphContext,
    render_context: &mut RenderContext,
    world: &World,
  ) -> Result<(), NodeRunError> {
    let handle = world.resource::<TextureHandle>();
    let gpu_images = world.resource::<RenderAssets<Image>>();
    if let Some(mesh_render_pipeline) = &self.mesh_render_pipeline.render_pipeline && let Some(direct_render_pipeline) = &self.direct_render_pipeline.render_pipeline && self.to_render && let Some(gpu_image) = gpu_images.get(&handle.0) {
      let meshes = world.resource::<RenderAssets<Mesh>>();
      let mesh_storage_assets = world.resource::<RenderAssets<GltfMeshStorage>>();
      let mesh_storage = world.resource::<GltfMeshStorageHandle>();
      let mesh_storage = &mesh_storage_assets[&mesh_storage.0];

      let mut meshes_to_render = vec![];

      let to_render = world.resource::<ItemRenderMap>();
      const RADIUS: f32 = 0.49;
      let mut vertex_buffer = vec![];
      let mut rendered_item_icons = HashMap::new();

      let mut add_block_to_vertices = |block_sprite: [BlockSprite; 6], x, y| {
        let top_tex   = block_sprite[4].into_uv();
        let left_tex  = block_sprite[0].into_uv();
        let right_tex = block_sprite[1].into_uv();
        let x = x * INVENTORY_OUTPUT_TEXTURE_WIDTH;
        let y = y * INVENTORY_OUTPUT_TEXTURE_WIDTH;
        vertex_buffer.extend_from_slice(&[HEX_NW[0] * RADIUS + x + 0.5, HEX_NW[1] * RADIUS + y + 0.5, top_tex.0[0], top_tex.0[1], 1.0]);
        vertex_buffer.extend_from_slice(&[HEX_NN[0] * RADIUS + x + 0.5, HEX_NN[1] * RADIUS + y + 0.5, top_tex.0[0], top_tex.1[1], 1.0]);
        vertex_buffer.extend_from_slice(&[HEX_CC[0] * RADIUS + x + 0.5, HEX_CC[1] * RADIUS + y + 0.5, top_tex.1[0], top_tex.0[1], 1.0]);
        vertex_buffer.extend_from_slice(&[HEX_NN[0] * RADIUS + x + 0.5, HEX_NN[1] * RADIUS + y + 0.5, top_tex.0[0], top_tex.1[1], 1.0]);
        vertex_buffer.extend_from_slice(&[HEX_CC[0] * RADIUS + x + 0.5, HEX_CC[1] * RADIUS + y + 0.5, top_tex.1[0], top_tex.0[1], 1.0]);
        vertex_buffer.extend_from_slice(&[HEX_NE[0] * RADIUS + x + 0.5, HEX_NE[1] * RADIUS + y + 0.5, top_tex.1[0], top_tex.1[1], 1.0]);

        vertex_buffer.extend_from_slice(&[HEX_NW[0] * RADIUS + x + 0.5, HEX_NW[1] * RADIUS + y + 0.5, left_tex.0[0], left_tex.0[1], 0.6]);
        vertex_buffer.extend_from_slice(&[HEX_SW[0] * RADIUS + x + 0.5, HEX_SW[1] * RADIUS + y + 0.5, left_tex.0[0], left_tex.1[1], 0.6]);
        vertex_buffer.extend_from_slice(&[HEX_CC[0] * RADIUS + x + 0.5, HEX_CC[1] * RADIUS + y + 0.5, left_tex.1[0], left_tex.0[1], 0.6]);
        vertex_buffer.extend_from_slice(&[HEX_SW[0] * RADIUS + x + 0.5, HEX_SW[1] * RADIUS + y + 0.5, left_tex.0[0], left_tex.1[1], 0.6]);
        vertex_buffer.extend_from_slice(&[HEX_CC[0] * RADIUS + x + 0.5, HEX_CC[1] * RADIUS + y + 0.5, left_tex.1[0], left_tex.0[1], 0.6]);
        vertex_buffer.extend_from_slice(&[HEX_SS[0] * RADIUS + x + 0.5, HEX_SS[1] * RADIUS + y + 0.5, left_tex.1[0], left_tex.1[1], 0.6]);

        vertex_buffer.extend_from_slice(&[HEX_CC[0] * RADIUS + x + 0.5, HEX_CC[1] * RADIUS + y + 0.5, right_tex.0[0], right_tex.0[1], 0.4]);
        vertex_buffer.extend_from_slice(&[HEX_SS[0] * RADIUS + x + 0.5, HEX_SS[1] * RADIUS + y + 0.5, right_tex.0[0], right_tex.1[1], 0.4]);
        vertex_buffer.extend_from_slice(&[HEX_NE[0] * RADIUS + x + 0.5, HEX_NE[1] * RADIUS + y + 0.5, right_tex.1[0], right_tex.0[1], 0.4]);
        vertex_buffer.extend_from_slice(&[HEX_SE[0] * RADIUS + x + 0.5, HEX_SE[1] * RADIUS + y + 0.5, right_tex.1[0], right_tex.1[1], 0.4]);
        vertex_buffer.extend_from_slice(&[HEX_SS[0] * RADIUS + x + 0.5, HEX_SS[1] * RADIUS + y + 0.5, right_tex.0[0], right_tex.1[1], 0.4]);
        vertex_buffer.extend_from_slice(&[HEX_NE[0] * RADIUS + x + 0.5, HEX_NE[1] * RADIUS + y + 0.5, right_tex.1[0], right_tex.0[1], 0.4]);
      };

      for (item, ItemRenderEntry { coord: (x, y), .. } ) in to_render.iter() {
        match item {
          BlockOrItem::Block(blockid) => {
            match blockid.deref_ext().render_info() {
              BlockRenderInfo::AsBlock(block_sprite) => {
                add_block_to_vertices(block_sprite, x, y);
              }
              BlockRenderInfo::AsMesh(mesh_handle) => {
                let mesh_handle = &mesh_storage[&mesh_handle].render.as_ref().unwrap();
                meshes_to_render.push((mesh_handle.clone(), [*x, *y, 0.0]));
              }
              BlockRenderInfo::Nothing => {}
              BlockRenderInfo::AsSkeleton(skeleton) => {
                for (_, mesh) in skeleton.to_skeleton_def().skeleton {
                  let mesh_handle = &mesh_storage[&mesh.mesh].render.as_ref().unwrap();
                  meshes_to_render.push((mesh_handle.clone(), [*x + mesh.offset.x / INVENTORY_OUTPUT_TEXTURE_WIDTH, *y - mesh.offset.y / INVENTORY_OUTPUT_TEXTURE_WIDTH, mesh.offset.z / INVENTORY_OUTPUT_TEXTURE_WIDTH]));
                }
              }
            }
          }
          BlockOrItem::Item(_) => {}
        }
        rendered_item_icons.insert(item.clone(), [x, y]);
      }

      let buffer = render_context
        .render_device
        .create_buffer_with_data(&BufferInitDescriptor {
          label: None,
          contents: &vertex_buffer.as_bytes(),
          usage: BufferUsages::VERTEX,
        });
      let contents = Mat4::orthographic_lh(0.0, INVENTORY_OUTPUT_TEXTURE_WIDTH as f32, INVENTORY_OUTPUT_TEXTURE_WIDTH as f32, 0.0, 0.01, 2.0);
      let view_buffer = render_context
        .render_device
        .create_buffer_with_data(&BufferInitDescriptor {
          label: None,
          contents: bytemuck::bytes_of(&contents),
          usage: BufferUsages::UNIFORM,
        });

      // TODO: this code is ass. Someone please fix it :(

      let direct_view_bind_group = self.direct_render_pipeline.create_view_bind_group(&render_context.render_device, &view_buffer);
      let direct_texture_bind_group = self.direct_render_pipeline.create_texture_bind_group(&render_context.render_device, gpu_image);

      let contents = Mat4::orthographic_lh(0.0, INVENTORY_OUTPUT_TEXTURE_WIDTH as f32, INVENTORY_OUTPUT_TEXTURE_WIDTH as f32, 0.0, 0.001, 4.0);
      let view_buffer = render_context
        .render_device
        .create_buffer_with_data(&BufferInitDescriptor {
          label: None,
          contents: bytemuck::bytes_of(&contents),
          usage: BufferUsages::UNIFORM,
        });

      let mut contents = vec![];
      for (_, [x, y, z]) in meshes_to_render.iter() {
        contents.extend_from_slice(
          &bytemuck::bytes_of(
            &(
              MatrixContent {
                mat: Mat4::from_translation(Vec3::new(0.5 + *x * INVENTORY_OUTPUT_TEXTURE_WIDTH, 0.5 + *y * INVENTORY_OUTPUT_TEXTURE_WIDTH,  1.0 + *z * INVENTORY_OUTPUT_TEXTURE_WIDTH)) *
                     Mat4::from_quat(Quat::from_euler(EulerRot::XYZ, -f32::FRAC_PI_4(), f32::FRAC_PI_4(), 0.0)) *
                     Mat4::from_scale(Vec3::new(0.55, -0.55, 0.55)),
                filler1: [0; 64],
                filler2: [0; 64],
                filler3: [0; 64],
              }
            )
          )
        );
      }

      let position_buffer = render_context
        .render_device
        .create_buffer_with_data(&BufferInitDescriptor {
          label: None,
          contents: bytemuck::cast_slice(contents.as_slice()),
          usage: BufferUsages::UNIFORM,
        });
      let mesh_position_bind_group = if contents.len() > 0 {
        Some(self.mesh_render_pipeline.create_position_bind_group(&render_context.render_device, &position_buffer))
      } else {
        None
      };

      let mesh_view_bind_group = self.mesh_render_pipeline.create_view_bind_group(&render_context.render_device, &view_buffer);
      let mesh_texture_bind_group = self.mesh_render_pipeline.create_texture_bind_group(&render_context.render_device, gpu_image);
      let mut pass = self.begin_render_pass(&mut render_context.command_encoder);

      pass.set_pipeline(direct_render_pipeline);
      let buf = &*buffer;
      pass.set_vertex_buffer(0, buf.slice(..));
      pass.set_bind_group(0, &direct_view_bind_group, &[]);
      pass.set_bind_group(1, &direct_texture_bind_group, &[]);
      pass.draw(0..(vertex_buffer.len()/5) as u32, 0..1);

      // Mesh pass

      pass.set_pipeline(mesh_render_pipeline);
      for (i, (handle, _)) in meshes_to_render.iter().enumerate() {
        let mesh = &meshes[handle];

        match &mesh.buffer_info {
          GpuBufferInfo::Indexed { buffer, count, .. } => {
            pass.set_vertex_buffer(0, *mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(*buffer.slice(..), IndexFormat::Uint32);
            pass.set_bind_group(0, &mesh_view_bind_group, &[]);
            pass.set_bind_group(1, &mesh_texture_bind_group, &[]);
            pass.set_bind_group(2, mesh_position_bind_group.as_ref().unwrap(), &[(i as u32) * 256]); // 256 is the minimum offset on M1. TODO: make it more generic
            pass.draw_indexed(0..*count, 0, 0..1);
          }
          _ => {}
        }
      }
    }
    graph
      .set_output(TEXTURE_NODE_OUTPUT_SLOT, SlotValue::TextureView(self.view.clone()))
      .unwrap();
    Ok(())
  }
}
