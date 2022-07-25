use bevy::ecs::system::lifetimeless::{Read, SQuery};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_phase::{EntityRenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass};
use bevy::render::render_resource::{
  BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState, ColorWrites,
  CompareFunction, DepthStencilState, Face, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
  RenderPipelineDescriptor, SamplerBindingType, ShaderStages, ShaderType, SpecializedRenderPipeline, TextureFormat,
  TextureSampleType, TextureViewDimension, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::ViewUniform;
use wgpu::BindGroupLayoutDescriptor;

use bevy::core_pipeline::Opaque3d;
use bevy::render::render_phase::AddRenderCommand;
use bevy::render::render_resource::SpecializedRenderPipelines;
use bevy::render::{RenderApp, RenderStage};

use crate::ecs::plugins::voxel::{
  extract_chunks, queue_chunks, ChunkMeshBuffer, ExtractedBlocks, LightTextureBindGroup, LightTextureHandle,
  RelightEvent, RemeshEvent, SelectionBindGroup, SetBindGroup, SetViewBindGroup, TextureHandle, VoxelTextureBindGroup,
  VoxelViewBindGroup,
};

pub struct VoxelPipeline {
  pub view_layout: BindGroupLayout,
  pub texture_layout: BindGroupLayout,
  pub selection_layout: BindGroupLayout,
  pub light_texture_layout: BindGroupLayout,
}

pub const VOXEL_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597899);
pub const VOXEL_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597999);

impl SpecializedRenderPipeline for VoxelPipeline {
  type Key = ();

  fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
    let shader_defs = Vec::new();
    let vertex_formats = vec![
      VertexFormat::Float32x3,
      VertexFormat::Float32x2,
      VertexFormat::Sint32x4,
      VertexFormat::Uint8x4,
    ];

    let vertex_layout = VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vertex_formats);

    RenderPipelineDescriptor {
      vertex: VertexState {
        shader: VOXEL_SHADER_VERTEX_HANDLE.typed::<Shader>(),
        entry_point: "main".into(),
        shader_defs: shader_defs.clone(),
        buffers: vec![vertex_layout],
      },
      fragment: Some(FragmentState {
        shader: VOXEL_SHADER_FRAGMENT_HANDLE.typed::<Shader>(),
        shader_defs,
        entry_point: "main".into(),
        targets: vec![ColorTargetState {
          format: TextureFormat::bevy_default(),
          blend: Some(BlendState::ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        }],
      }),
      layout: Some(vec![
        self.view_layout.clone(),
        self.texture_layout.clone(),
        self.selection_layout.clone(),
        self.light_texture_layout.clone(),
      ]),
      primitive: PrimitiveState {
        front_face: FrontFace::Ccw,
        cull_mode: Some(Face::Front),
        unclipped_depth: false,
        polygon_mode: PolygonMode::Fill,
        conservative: false,
        topology: PrimitiveTopology::TriangleList,
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
      label: Some("voxel_pipeline".into()),
    }
  }
}

impl FromWorld for VoxelPipeline {
  fn from_world(world: &mut World) -> Self {
    let render_device = world.resource::<RenderDevice>();
    VoxelPipeline {
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
        label: Some("texture_layout"),
      }),
      selection_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
        label: Some("selection_layout"),
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

pub type DrawVoxelsFull = (
  SetItemPipeline,
  SetViewBindGroup<0, VoxelViewBindGroup>,
  SetBindGroup<1, VoxelTextureBindGroup>,
  SetBindGroup<2, SelectionBindGroup>,
  SetBindGroup<3, LightTextureBindGroup>,
  DrawVoxels,
);

pub struct DrawVoxels;
impl EntityRenderCommand for DrawVoxels {
  type Param = SQuery<Read<ChunkMeshBuffer>>;

  fn render<'w>(
    _view: Entity,
    item: Entity,
    param: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let ChunkMeshBuffer(buf, verticies) = param.get_inner(item).unwrap();
    pass.set_vertex_buffer(0, buf.slice(..));
    pass.draw(0..*verticies as u32 * 6, 0..1 as u32);
    RenderCommandResult::Success
  }
}

pub struct VoxelRendererPlugin;

impl Plugin for VoxelRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let voxel_shader_vertex = Shader::from_spirv(include_bytes!("../../../../assets/shader/voxel.vert.spv").as_slice());
    let voxel_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../assets/shader/voxel.frag.spv").as_slice());
    shaders.set_untracked(VOXEL_SHADER_VERTEX_HANDLE, voxel_shader_vertex);
    shaders.set_untracked(VOXEL_SHADER_FRAGMENT_HANDLE, voxel_shader_fragment);

    app.add_event::<RemeshEvent>();
    app.add_event::<RelightEvent>();

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app
      .init_resource::<ExtractedBlocks>()
      .init_resource::<VoxelPipeline>()
      .init_resource::<SpecializedRenderPipelines<VoxelPipeline>>()
      .init_resource::<TextureHandle>()
      .init_resource::<LightTextureHandle>()
      .init_resource::<VoxelViewBindGroup>()
      .init_resource::<SelectionBindGroup>()
      .init_resource::<VoxelTextureBindGroup>()
      .init_resource::<LightTextureBindGroup>()
      .add_system_to_stage(RenderStage::Extract, extract_chunks)
      .add_system_to_stage(RenderStage::Queue, queue_chunks)
      .add_render_command::<Opaque3d, DrawVoxelsFull>();
  }
}
