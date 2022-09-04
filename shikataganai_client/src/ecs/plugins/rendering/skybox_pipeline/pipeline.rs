use bevy::pbr::{MeshUniform, MESH_SHADER_HANDLE};
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_resource::ShaderType;
use bevy::render::render_resource::{
  BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState, ColorWrites,
  CompareFunction, DepthStencilState, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
  RenderPipelineDescriptor, SamplerBindingType, ShaderStages, SpecializedRenderPipeline, TextureFormat,
  TextureSampleType, TextureViewDimension, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::ViewUniform;
use bevy_atmosphere::skybox::ATMOSPHERE_SKYBOX_SHADER_HANDLE;
use wgpu::BindGroupLayoutDescriptor;

pub struct SkyboxPipeline {
  pub view_layout: BindGroupLayout,
  pub texture_layout: BindGroupLayout,
  pub mesh_layout: BindGroupLayout,
}

impl SpecializedRenderPipeline for SkyboxPipeline {
  type Key = ();

  fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
    let vertex_formats = vec![VertexFormat::Float32x3, VertexFormat::Float32x3];

    let vertex_layout = VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vertex_formats);

    RenderPipelineDescriptor {
      vertex: VertexState {
        shader: MESH_SHADER_HANDLE.typed::<Shader>(),
        entry_point: "vertex".into(),
        shader_defs: vec![],
        buffers: vec![vertex_layout],
      },
      fragment: Some(FragmentState {
        shader: ATMOSPHERE_SKYBOX_SHADER_HANDLE.typed::<Shader>(),
        shader_defs: vec![],
        entry_point: "fragment".into(),
        targets: vec![Some(ColorTargetState {
          format: TextureFormat::bevy_default(),
          blend: Some(BlendState::ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        })],
      }),
      layout: Some(vec![
        self.view_layout.clone(),
        self.texture_layout.clone(),
        self.mesh_layout.clone(),
      ]),
      primitive: PrimitiveState {
        front_face: FrontFace::Ccw,
        cull_mode: None,
        unclipped_depth: true,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
        topology: PrimitiveTopology::TriangleList,
        strip_index_format: None,
      },
      depth_stencil: Some(DepthStencilState {
        format: TextureFormat::Depth32Float,
        depth_write_enabled: false,
        depth_compare: CompareFunction::Always,
        stencil: Default::default(),
        bias: Default::default(),
      }),
      multisample: MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      label: Some("skybox_pipeline".into()),
    }
  }
}

impl FromWorld for SkyboxPipeline {
  fn from_world(world: &mut World) -> Self {
    let render_device = world.resource::<RenderDevice>();

    let mesh_binding = BindGroupLayoutEntry {
      binding: 0,
      visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
      ty: BindingType::Buffer {
        ty: BufferBindingType::Uniform,
        has_dynamic_offset: true,
        min_binding_size: Some(MeshUniform::min_size()),
      },
      count: None,
    };

    let mesh_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
      entries: &[mesh_binding],
      label: Some("mesh_layout"),
    });

    SkyboxPipeline {
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
      mesh_layout,
      texture_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[
          BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
              multisampled: false,
              sample_type: TextureSampleType::Float { filterable: true },
              view_dimension: TextureViewDimension::Cube,
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
        label: Some("texture_layout"),
      }),
    }
  }
}
