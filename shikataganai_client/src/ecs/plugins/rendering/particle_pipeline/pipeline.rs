use crate::ecs::plugins::rendering::voxel_pipeline::{VOXEL_SHADER_FRAGMENT_HANDLE, VOXEL_SHADER_VERTEX_HANDLE};
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_resource::ShaderType;
use bevy::render::render_resource::{
  BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState, ColorWrites,
  CompareFunction, DepthStencilState, Face, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
  RenderPipelineDescriptor, SamplerBindingType, ShaderStages, SpecializedRenderPipeline, TextureFormat,
  TextureSampleType, TextureViewDimension, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::ViewUniform;
use wgpu::BindGroupLayoutDescriptor;
use crate::ecs::plugins::rendering::particle_pipeline::{PARTICLE_SHADER_FRAGMENT_HANDLE, PARTICLE_SHADER_VERTEX_HANDLE};

#[derive(Resource)]
pub struct ParticlePipeline {
  pub view_layout: BindGroupLayout,
  pub texture_layout: BindGroupLayout,
  pub aspect_ratio_layout: BindGroupLayout,
  pub light_texture_layout: BindGroupLayout,
}

impl SpecializedRenderPipeline for ParticlePipeline {
  type Key = ();

  fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
    let shader_defs = Vec::new();
    let vertex_formats = vec![
      VertexFormat::Float32x3,
      VertexFormat::Sint32,
      VertexFormat::Sint32,
      VertexFormat::Uint32,
    ];

    let vertex_layout = VertexBufferLayout::from_vertex_formats(VertexStepMode::Instance, vertex_formats);

    RenderPipelineDescriptor {
      vertex: VertexState {
        shader: PARTICLE_SHADER_VERTEX_HANDLE.typed::<Shader>(),
        entry_point: "main".into(),
        shader_defs: shader_defs.clone(),
        buffers: vec![vertex_layout],
      },
      fragment: Some(FragmentState {
        shader: PARTICLE_SHADER_FRAGMENT_HANDLE.typed::<Shader>(),
        shader_defs,
        entry_point: "main".into(),
        targets: vec![Some(ColorTargetState {
          format: TextureFormat::bevy_default(),
          blend: Some(BlendState::ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        })],
      }),
      layout: Some(vec![
        self.view_layout.clone(),
        self.texture_layout.clone(),
        self.aspect_ratio_layout.clone(),
        self.light_texture_layout.clone(),
      ]),
      primitive: PrimitiveState {
        front_face: FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
        topology: PrimitiveTopology::TriangleStrip,
        strip_index_format: None,
      },
      depth_stencil: Some(DepthStencilState {
        format: TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: CompareFunction::GreaterEqual,
        stencil: Default::default(),
        bias: Default::default(),
      }),
      multisample: MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      label: Some("particle_pipeline".into()),
    }
  }
}

impl FromWorld for ParticlePipeline {
  fn from_world(world: &mut World) -> Self {
    let render_device = world.resource::<RenderDevice>();
    Self {
      view_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: true,
            min_binding_size: Some(ViewUniform::min_size()),
          },
          count: None,
        }],
        label: Some("view_layout"),
      }),
      texture_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
          BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
              multisampled: false,
              sample_type: TextureSampleType::Float { filterable: true },
              view_dimension: TextureViewDimension::D2Array,
            },
            count: None,
          },
          BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
          },
        ]
      }),
      aspect_ratio_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: None,
        entries: &[
          BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
              ty: BufferBindingType::Uniform,
              has_dynamic_offset: false,
              min_binding_size: None,
            },
            count: None,
          },
        ]
      }),
      light_texture_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[
          BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Texture {
              multisampled: false,
              sample_type: TextureSampleType::Float { filterable: true },
              view_dimension: TextureViewDimension::D2,
            },
            count: None,
          },
          BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
          },
        ],
        label: Some("light_texture_layout"),
      }),
    }
  }
}
