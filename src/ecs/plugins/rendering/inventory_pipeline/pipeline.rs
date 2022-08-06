use crate::ecs::plugins::rendering::inventory_pipeline::{
  INVENTORY_SHADER_FRAGMENT_HANDLE, INVENTORY_SHADER_VERTEX_HANDLE,
};
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_resource::{
  BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType,
  CachedRenderPipelineId, ColorTargetState, ColorWrites, Extent3d, FragmentState, FrontFace, MultisampleState,
  PipelineCache, PolygonMode, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
  ShaderStages, TextureAspect, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView,
  TextureViewDimension, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::{BevyDefault, GpuImage};
use wgpu::{
  BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindingResource, CommandEncoder, LoadOp, Operations,
  RenderPass, RenderPassColorAttachment, RenderPassDescriptor, TextureDescriptor, TextureViewDescriptor,
};

pub struct InventoryNode {
  pub render_pipeline: Option<RenderPipeline>,
  pub view_layout: BindGroupLayout,
  pub texture_layout: BindGroupLayout,
  pub render_pipeline_id: CachedRenderPipelineId,
  pub view: TextureView,
  pub to_render: bool,
  pub initialised: bool,
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

    let vertex_formats = vec![VertexFormat::Float32x2, VertexFormat::Float32x2, VertexFormat::Float32];

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
      to_render: false,
      view: InventoryNode::create_view(render_device),
      initialised: false,
    }
  }

  pub fn create_view(render_device: &RenderDevice) -> TextureView {
    let texture = render_device.create_texture(&TextureDescriptor {
      label: "offscreen_texture".into(),
      size: Extent3d {
        width: 1024,
        height: 1024,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: TextureDimension::D2,
      format: TextureFormat::bevy_default(),
      usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
    });
    texture.create_view(&TextureViewDescriptor {
      label: "offscreen_texture_view".into(),
      format: Some(TextureFormat::bevy_default()),
      dimension: Some(TextureViewDimension::D2),
      aspect: TextureAspect::All,
      base_mip_level: 0,
      mip_level_count: None,
      base_array_layer: 0,
      array_layer_count: None,
    })
  }

  pub fn create_view_bind_group(&self, render_device: &RenderDevice, view_buffer: &Buffer) -> BindGroup {
    render_device.create_bind_group(&BindGroupDescriptor {
      label: None,
      layout: &self.view_layout,
      entries: &[BindGroupEntry {
        binding: 0,
        resource: BindingResource::Buffer(view_buffer.as_entire_buffer_binding()),
      }],
    })
  }

  pub fn create_texture_bind_group(&self, render_device: &RenderDevice, gpu_image: &GpuImage) -> BindGroup {
    render_device.create_bind_group(&BindGroupDescriptor {
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
    })
  }

  pub fn begin_render_pass<'a>(&'a self, command_encoder: &'a mut CommandEncoder) -> RenderPass<'a> {
    command_encoder.begin_render_pass(&RenderPassDescriptor {
      label: Some("offscreen_pass"),
      color_attachments: &[Some(RenderPassColorAttachment {
        view: &self.view,
        resolve_target: None,
        ops: Operations {
          load: LoadOp::Clear(wgpu::Color {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 0.0,
          }),
          store: true,
        },
      })],
      depth_stencil_attachment: None,
    })
  }
}
