use crate::ecs::plugins::rendering::inventory_pipeline::{
  INVENTORY_SHADER_FRAGMENT_HANDLE, INVENTORY_SHADER_VERTEX_HANDLE,
};
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_resource::{
  BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, CachedRenderPipelineId,
  ColorTargetState, ColorWrites, FragmentState, FrontFace, MultisampleState, PipelineCache, PolygonMode,
  PrimitiveState, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, TextureFormat,
  TextureSampleType, TextureViewDimension, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use wgpu::BindGroupLayoutDescriptor;

pub struct InventoryNode {
  pub render_pipeline: Option<RenderPipeline>,
  pub view_layout: BindGroupLayout,
  pub texture_layout: BindGroupLayout,
  pub render_pipeline_id: CachedRenderPipelineId,
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
