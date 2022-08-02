use crate::ecs::plugins::rendering::inventory_pipeline::pipeline::InventoryNode;
use crate::ecs::plugins::rendering::inventory_pipeline::TEXTURE_NODE_OUTPUT_SLOT;
use crate::ecs::plugins::rendering::voxel_pipeline::bind_groups::TextureHandle;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType, SlotValue};
use bevy::render::render_resource::{
  BufferUsages, Extent3d, PipelineCache, TextureAspect, TextureDimension, TextureFormat, TextureUsages,
  TextureViewDimension,
};
use bevy::render::renderer::RenderContext;
use bevy::render::texture::BevyDefault;
use wgpu::util::BufferInitDescriptor;
use wgpu::{
  BindGroupDescriptor, BindGroupEntry, BindingResource, LoadOp, Operations, RenderPassColorAttachment,
  RenderPassDescriptor, TextureDescriptor, TextureViewDescriptor,
};

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
    self.render_pipeline = world
      .resource_mut::<PipelineCache>()
      .get_render_pipeline(self.render_pipeline_id)
      .cloned();
  }
  fn run(
    &self,
    graph: &mut RenderGraphContext,
    render_context: &mut RenderContext,
    world: &World,
  ) -> Result<(), NodeRunError> {
    if let Some(render_pipeline) = &self.render_pipeline {
      let texture = render_context.render_device.create_texture(&TextureDescriptor {
        label: "offscreen_texture".into(),
        size: Extent3d {
          width: 256,
          height: 256,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::bevy_default(),
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
      });
      let view = texture.create_view(&TextureViewDescriptor {
        label: "offscreen_texture_view".into(),
        format: Some(TextureFormat::bevy_default()),
        dimension: Some(TextureViewDimension::D2),
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
      });

      let contents: [[f32; 5]; 6] = [
        [-1.0, -1.0, 1.0, 0.0, 1.0],
        [-1.0, 1.0, 1.0, 0.0, 0.0],
        [1.0, -1.0, 1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0, 0.0, 0.0],
        [1.0, -1.0, 1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0, 1.0, 0.0],
      ];
      let buffer = render_context
        .render_device
        .create_buffer_with_data(&BufferInitDescriptor {
          label: None,
          contents: bytemuck::bytes_of(&contents),
          usage: BufferUsages::VERTEX,
        });
      let contents = Mat4::orthographic_lh(-1.0, 1.0, -1.0, 1.0, 0.01, 2.0);
      let view_buffer = render_context
        .render_device
        .create_buffer_with_data(&BufferInitDescriptor {
          label: None,
          contents: bytemuck::bytes_of(&contents),
          usage: BufferUsages::UNIFORM,
        });
      let view_bind_group = render_context.render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &self.view_layout,
        entries: &[BindGroupEntry {
          binding: 0,
          resource: BindingResource::Buffer(view_buffer.as_entire_buffer_binding()),
        }],
      });

      let handle = world.resource::<TextureHandle>();
      let gpu_images = world.resource::<RenderAssets<Image>>();
      let texture_bind_group = if let Some(gpu_image) = gpu_images.get(&handle.0) {
        Some(render_context.render_device.create_bind_group(&BindGroupDescriptor {
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
          label: Some("inventory_texture_bind_group"),
          layout: &self.texture_layout,
        }))
      } else {
        None
      };
      if let Some(texture_bind_group) = texture_bind_group {
        let mut pass = render_context.command_encoder.begin_render_pass(&RenderPassDescriptor {
          label: Some("offscreen_pass"),
          color_attachments: &[Some(RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: Operations {
              load: LoadOp::Clear(wgpu::Color {
                r: 1.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
              }),
              store: true,
            },
          })],
          depth_stencil_attachment: None,
        });
        pass.set_pipeline(render_pipeline);
        let buf = &*buffer;
        pass.set_vertex_buffer(0, buf.slice(..));
        pass.set_bind_group(0, &view_bind_group, &[]);
        pass.set_bind_group(1, &texture_bind_group, &[]);
        pass.draw(0..6, 0..1)
      }
      graph
        .set_output(TEXTURE_NODE_OUTPUT_SLOT, SlotValue::TextureView(view))
        .unwrap();
    }
    Ok(())
  }
}
