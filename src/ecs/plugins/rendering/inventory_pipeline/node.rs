use crate::ecs::components::block::BlockId;
use crate::ecs::components::block_or_item::BlockOrItem;
use crate::ecs::plugins::rendering::inventory_pipeline::pipeline::InventoryNode;
use crate::ecs::plugins::rendering::inventory_pipeline::{
  ExtractedItems, INVENTORY_OUTPUT_TEXTURE_WIDTH, TEXTURE_NODE_OUTPUT_SLOT,
};
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::TextureHandle;
use crate::ecs::resources::player::RerenderInventory;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType, SlotValue};
use bevy::render::render_resource::{BufferUsages, PipelineCache};
use bevy::render::renderer::{RenderContext, RenderDevice};
use image::EncodableLayout;
use std::collections::HashMap;
use wgpu::util::BufferInitDescriptor;

const HEX_CC: [f32; 2] = [0.000000, -0.000000];
const HEX_NN: [f32; 2] = [0.000000, -1.000000];
const HEX_SS: [f32; 2] = [0.000000, 1.000000];
const HEX_NW: [f32; 2] = [-0.866025, -0.500000];
const HEX_NE: [f32; 2] = [0.866025, -0.500000];
const HEX_SW: [f32; 2] = [-0.866025, 0.500000];
const HEX_SE: [f32; 2] = [0.866025, 0.500000];

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
    if world.resource::<RerenderInventory>().0 || !self.initialised {
      self.to_render = true;
      self.view = InventoryNode::create_view(world.resource::<RenderDevice>());
    } else {
      if self.initialised {
        self.to_render = false;
      }
    }

    self.render_pipeline = world
      .resource_mut::<PipelineCache>()
      .get_render_pipeline(self.render_pipeline_id)
      .cloned();
    let handle = world.resource::<TextureHandle>();
    let gpu_images = world.resource::<RenderAssets<Image>>();
    self.initialised = self.render_pipeline.is_some() && gpu_images.get(&handle.0).is_some();
  }
  fn run(
    &self,
    graph: &mut RenderGraphContext,
    render_context: &mut RenderContext,
    world: &World,
  ) -> Result<(), NodeRunError> {
    let handle = world.resource::<TextureHandle>();
    let gpu_images = world.resource::<RenderAssets<Image>>();
    if let Some(render_pipeline) = &self.render_pipeline && self.to_render && let Some(gpu_image) = gpu_images.get(&handle.0) {
      let to_render = world.resource::<ExtractedItems>();
      const RADIUS: f32 = 0.49;
      let mut vertex_buffer = vec![];
      let mut rendered_item_icons = HashMap::new();

      let mut add_block_to_vertices = |block: &BlockId, x, y| {
        let top_tex   = block.into_array_of_faces()[4].into_uv();
        let left_tex  = block.into_array_of_faces()[0].into_uv();
        let right_tex = block.into_array_of_faces()[1].into_uv();
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

      for (item, (x, y)) in to_render.0.iter() {
        match item {
          BlockOrItem::Block(blockid) => {
            add_block_to_vertices(blockid, x, y);
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

      let view_bind_group = self.create_view_bind_group(&render_context.render_device, &view_buffer);
      let texture_bind_group = self.create_texture_bind_group(&render_context.render_device, gpu_image);
      let mut pass = self.begin_render_pass(&mut render_context.command_encoder);

      pass.set_pipeline(render_pipeline);
      let buf = &*buffer;
      pass.set_vertex_buffer(0, buf.slice(..));
      pass.set_bind_group(0, &view_bind_group, &[]);
      pass.set_bind_group(1, &texture_bind_group, &[]);
      pass.draw(0..(vertex_buffer.len()/5) as u32, 0..1)
    }
    graph
      .set_output(TEXTURE_NODE_OUTPUT_SLOT, SlotValue::TextureView(self.view.clone()))
      .unwrap();
    Ok(())
  }
}
