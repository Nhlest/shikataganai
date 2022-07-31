use crate::ecs::plugins::imgui::{IMGUI_PASS, TEXTURE_NODE_INPUT_SLOT};
use crate::ecs::plugins::voxel::TextureHandle;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, SlotInfo, SlotType, SlotValue};
use bevy::render::render_resource::{
  BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, BufferUsages,
  CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d, FragmentState, FrontFace, MultisampleState,
  PipelineCache, PolygonMode, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
  ShaderStages, TextureAspect, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDimension,
  VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::BevyDefault;
use bevy::render::RenderApp;
use std::ops::{Deref, DerefMut};
use wgpu::util::BufferInitDescriptor;
use wgpu::{
  BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindingResource, LoadOp, Operations,
  RenderPassColorAttachment, RenderPassDescriptor, TextureDescriptor, TextureViewDescriptor,
};

pub struct OffscreenInventoryAuxRendererPlugin;

pub const INVENTORY_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151595899);
pub const INVENTORY_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151596999);

pub struct InventoryNode {
  render_pipeline: Option<RenderPipeline>,
  view_layout: BindGroupLayout,
  texture_layout: BindGroupLayout,
  render_pipeline_id: CachedRenderPipelineId,
}

impl InventoryNode {
  pub fn new(render_device: &RenderDevice, render_pipeline_cache: &mut PipelineCache) -> Self {
    let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      label: None,
      entries: &[BindGroupLayoutEntry {
        binding: 0,
        visibility: ShaderStages::VERTEX,
        ty: BindingType::Buffer {
          ty: BufferBindingType::Uniform,
          has_dynamic_offset: false,
          min_binding_size: None,
        },
        count: None,
      }],
    });

    let texture_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[
        BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Texture {
            multisampled: false,
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
          },
          count: None,
        },
        BindGroupLayoutEntry {
          binding: 1,
          visibility: ShaderStages::FRAGMENT,
          ty: BindingType::Sampler(SamplerBindingType::Filtering),
          count: None,
        },
      ],
      label: Some("inventory_texture_layout"),
    });

    let vertex_formats = vec![VertexFormat::Float32x3, VertexFormat::Float32x2];

    let vertex_layout = VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vertex_formats);

    let render_pipeline_descriptor = RenderPipelineDescriptor {
      vertex: VertexState {
        shader: INVENTORY_SHADER_VERTEX_HANDLE.typed::<Shader>(),
        entry_point: "main".into(),
        shader_defs: vec![],
        buffers: vec![vertex_layout],
      },
      fragment: Some(FragmentState {
        shader: INVENTORY_SHADER_FRAGMENT_HANDLE.typed::<Shader>(),
        shader_defs: vec![],
        entry_point: "main".into(),
        targets: vec![Some(ColorTargetState {
          format: TextureFormat::bevy_default(),
          blend: Some(BlendState::ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        })],
      }),
      layout: Some(vec![view_layout.clone(), texture_layout.clone()]),
      primitive: PrimitiveState {
        front_face: FrontFace::Cw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
      },
      depth_stencil: None,
      multisample: MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      label: Some("inventory_pipeline_layout".into()),
    };

    let render_pipeline_id = render_pipeline_cache.queue_render_pipeline(render_pipeline_descriptor);

    Self {
      view_layout,
      texture_layout,
      render_pipeline_id,
      render_pipeline: None,
    }
  }
}

pub const TEXTURE_NODE_OUTPUT_SLOT: &'static str = "Texture Slot Output";
pub const INVENTORY_PASS: &'static str = "Inventory Pass";

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

impl Plugin for OffscreenInventoryAuxRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let voxel_shader_vertex =
      Shader::from_spirv(include_bytes!("../../../../shaders/output/offscreen.vert.spv").as_slice());
    let voxel_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../shaders/output/offscreen.frag.spv").as_slice());
    shaders.set_untracked(INVENTORY_SHADER_VERTEX_HANDLE, voxel_shader_vertex);
    shaders.set_untracked(INVENTORY_SHADER_FRAGMENT_HANDLE, voxel_shader_fragment);

    if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
      let inventory_node = {
        let world_cell = render_app.world.cell();
        let mut render_pipeline_cache = world_cell.resource_mut::<PipelineCache>();
        let render_device = world_cell.resource::<RenderDevice>();
        InventoryNode::new(render_device.deref(), render_pipeline_cache.deref_mut())
      };

      let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
      render_graph.add_node(INVENTORY_PASS, inventory_node);

      render_graph.add_node_edge(INVENTORY_PASS, IMGUI_PASS).unwrap();
      render_graph
        .add_slot_edge(
          INVENTORY_PASS,
          TEXTURE_NODE_OUTPUT_SLOT,
          IMGUI_PASS,
          TEXTURE_NODE_INPUT_SLOT,
        )
        .unwrap();
    }
  }
}
