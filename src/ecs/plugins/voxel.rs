use bevy::core_pipeline::Opaque3d;
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{
  AddRenderCommand, DrawFunctions, EntityRenderCommand, PhaseItem, RenderCommand, RenderCommandResult, RenderPhase,
  SetItemPipeline, TrackedRenderPass,
};
use bevy::render::render_resource::{
  BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, BufferVec,
  ColorTargetState, ColorWrites, CompareFunction, FragmentState, FrontFace, MultisampleState, PipelineCache,
  PolygonMode, PrimitiveState, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, SpecializedRenderPipeline,
  SpecializedRenderPipelines, TextureFormat, TextureSampleType, TextureViewDimension, VertexBufferLayout, VertexFormat,
  VertexState, VertexStepMode,
};
use bevy::render::render_resource::{Buffer, ShaderType};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::texture::BevyDefault;
use bevy::render::view::{ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::render::RenderStage;
use bevy::render::{RenderApp, RenderWorld};
use bevy::utils::hashbrown::HashMap;
use bytemuck_derive::*;
use itertools::Itertools;
use wgpu::util::BufferInitDescriptor;
use wgpu::{
  BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindingResource, BufferUsages, DepthStencilState,
  Face,
};

use crate::ecs::components::block::Block;
use crate::ecs::plugins::camera::Selection;
use crate::ecs::resources::block::BlockSprite;
use crate::ecs::resources::chunk_map::BlockAccessor;
use crate::ecs::resources::chunk_map::BlockAccessorStatic;
use crate::util::array::{ArrayIndex, ImmediateNeighbours, DD, DDD};

pub struct VoxelRendererPlugin;

pub enum RelightType {
  LightSourceAdded,
  LightSourceRemoved,
  BlockAdded,
  BlockRemoved,
}

pub enum RemeshEvent {
  Remesh(DD),
}

pub enum RelightEvent {
  Relight(RelightType, DDD),
}

fn extract_chunks(
  mut render_world: ResMut<RenderWorld>,
  mut block_accessor: BlockAccessorStatic,
  selection: Res<Option<Selection>>,
  mut remesh_events: EventReader<RemeshEvent>,
) {
  render_world.insert_resource(selection.clone());
  let mut updated = vec![];

  for ch in remesh_events
    .iter()
    .filter_map(|p| if let RemeshEvent::Remesh(d) = p { Some(d) } else { None })
    .unique()
  {
    if !block_accessor.chunk_map.map.contains_key(ch) {
      continue;
    }
    updated.push(*ch);
    let mut extracted_blocks = render_world.get_resource_mut::<ExtractedBlocks>().unwrap();
    extracted_blocks
      .blocks
      .insert(*ch, BufferVec::new(BufferUsages::VERTEX));
    let entity = block_accessor.chunk_map.map[ch].entity.unwrap();
    let bounds = block_accessor.chunks.get(entity).unwrap().grid.bounds;
    let mut i = bounds.0;
    loop {
      let block: Block = block_accessor.get_single(i).unwrap().clone();
      if block.visible() {
        for neighbour in i.immeidate_neighbours() {
          if block_accessor.get_single(neighbour).map_or(true, |b| !b.visible()) {
            let light_level = block_accessor.get_light_level(neighbour);
            let lighting = match light_level {
              Some(light_level) => (light_level.heaven, light_level.hearth),
              None => (0, 0),
            };

            extracted_blocks.blocks.get_mut(ch).unwrap().push(SingleSide::new(
              (i.0 as f32, i.1 as f32, i.2 as f32),
              (neighbour.0 - i.0, neighbour.1 - i.1, neighbour.2 - i.2),
              block.block.into_array_of_faces(),
              lighting,
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
  render_world.insert_resource(updated);
}

impl SingleSide {
  fn new(
    (x, y, z): (f32, f32, f32),
    (ix, iy, iz): (i32, i32, i32),
    block: [BlockSprite; 6],
    lighting: (u8, u8),
  ) -> Self {
    let fx = x;
    let fy = y;
    let fz = z;
    let side = if ix != 0 {
      if ix == 1 {
        0
      } else {
        1
      }
    } else if iz != 0 {
      if iz == 1 {
        2
      } else {
        3
      }
    } else {
      if iy == 1 {
        4
      } else {
        5
      }
    };
    let triangles = VERTEX[side];
    SingleSide(triangles.map(
      |Vertex {
         pos: [vx, vy, vz],
         uv: [uv0, uv1],
       }| SingleVertex {
        position: [vx + fx, vy + fy, vz + fz],
        uv: [
          uv0 / 8.0 + block[side].into_uv().0[0],
          uv1 / 8.0 + block[side].into_uv().0[1],
        ],
        tile_side: [x.floor() as i32, y.floor() as i32, z.floor() as i32, side as i32],
        meta: [lighting.0, lighting.1, 0, 0],
      },
    ))
  }
}

pub struct VoxelPipeline {
  view_layout: BindGroupLayout,
  texture_layout: BindGroupLayout,
  selection_layout: BindGroupLayout,
  light_texture_layout: BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SingleVertex {
  pub position: [f32; 3],
  pub uv: [f32; 2],
  pub tile_side: [i32; 4],
  pub meta: [u8; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SingleSide([SingleVertex; 6]);

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
      label: Some("chunk_pipeline".into()),
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
        label: Some("chunk_view_layout"),
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
        label: Some("chunk_view_layout"),
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

pub struct TextureHandle(Handle<Image>);
pub struct LightTextureHandle(Handle<Image>);

impl FromWorld for TextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    TextureHandle(asset_server.load("texture.png"))
  }
}

impl FromWorld for LightTextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    LightTextureHandle(asset_server.load("light.png"))
  }
}

fn queue_chunks(
  mut commands: Commands,
  mut extracted_blocks: ResMut<ExtractedBlocks>,
  mut views: Query<&mut RenderPhase<Opaque3d>>,
  draw_functions: Res<DrawFunctions<Opaque3d>>,
  mut pipelines: ResMut<SpecializedRenderPipelines<VoxelPipeline>>,
  mut pipeline_cache: ResMut<PipelineCache>,
  chunk_pipeline: Res<VoxelPipeline>,
  (render_device, render_queue): (Res<RenderDevice>, Res<RenderQueue>),
  view_uniforms: Res<ViewUniforms>,
  mut voxel_bind_group: ResMut<VoxelViewBindGroup>,
  mut selection_bind_group: ResMut<SelectionBindGroup>,
  gpu_images: Res<RenderAssets<Image>>,
  (handle, light_texture_handle, mut bind_group, mut light_texture_bind_group): (
    Res<TextureHandle>,
    Res<LightTextureHandle>,
    ResMut<TextureBindGroup>,
    ResMut<LightTextureBindGroup>,
  ),
  selection: Res<Option<Selection>>,
  updated: Res<Vec<DD>>,
) {
  if let Some(gpu_image) = gpu_images.get(&handle.0) {
    *bind_group = TextureBindGroup {
      bind_group: Some(render_device.create_bind_group(&BindGroupDescriptor {
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
      })),
    };
  }
  if let Some(gpu_image) = gpu_images.get(&light_texture_handle.0) {
    *light_texture_bind_group = LightTextureBindGroup {
      bind_group: Some(render_device.create_bind_group(&BindGroupDescriptor {
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
      })),
    };
  }

  if let Some(view_binding) = view_uniforms.uniforms.binding() {
    voxel_bind_group.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
      entries: &[BindGroupEntry {
        binding: 0,
        resource: view_binding,
      }],
      label: Some("block_view_bind_group"),
      layout: &chunk_pipeline.view_layout,
    }));
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

  selection_bind_group.bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
    entries: &[BindGroupEntry {
      binding: 0,
      resource: BindingResource::Buffer(selection_buffer.as_entire_buffer_binding()),
    }],
    label: Some("block_view_bind_group"),
    layout: &chunk_pipeline.selection_layout,
  }));

  let draw_function = draw_functions.read().get_id::<DrawChunkFull>().unwrap();

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
        .insert(MeshBuffer(buf.buffer().unwrap().clone(), buf.len()))
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

type DrawChunkFull = (
  SetItemPipeline,
  SetVoxelViewBindGroup<0>,
  SetVoxelTextureBindGroup<1>,
  SetSelectionBindGroup<2>,
  SetLightTextureBindGroup<3>,
  DrawChunk,
);

#[derive(Default)]
pub struct VoxelViewBindGroup {
  bind_group: Option<BindGroup>,
}

pub struct SetVoxelViewBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetVoxelViewBindGroup<I> {
  type Param = (SRes<VoxelViewBindGroup>, SQuery<Read<ViewUniformOffset>>);

  fn render<'w>(
    view: Entity,
    _item: &P,
    (bind_group, view_query): SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let view_uniform = view_query.get(view).unwrap();
    pass.set_bind_group(
      I,
      &bind_group.into_inner().bind_group.as_ref().unwrap(),
      &[view_uniform.offset],
    );
    RenderCommandResult::Success
  }
}

#[derive(Default)]
pub struct TextureBindGroup {
  bind_group: Option<BindGroup>,
}

pub struct SetVoxelTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetVoxelTextureBindGroup<I> {
  type Param = SRes<TextureBindGroup>;

  fn render<'w>(
    _view: Entity,
    _item: Entity,
    texture_bind_group: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    if let Some(texture_bind_group) = texture_bind_group.into_inner().bind_group.as_ref() {
      pass.set_bind_group(I, texture_bind_group, &[]);
      RenderCommandResult::Success
    } else {
      RenderCommandResult::Failure
    }
  }
}

#[derive(Default)]
pub struct LightBindGroup {
  bind_group: Option<BindGroup>,
}

pub struct SetLightBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetLightBindGroup<I> {
  type Param = SRes<LightBindGroup>;

  fn render<'w>(
    _view: Entity,
    _item: Entity,
    light_bind_group: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    if let Some(light_bind_group) = light_bind_group.into_inner().bind_group.as_ref() {
      pass.set_bind_group(I, light_bind_group, &[]);
      RenderCommandResult::Success
    } else {
      RenderCommandResult::Failure
    }
  }
}

struct DrawChunk;

#[derive(Component)]
struct MeshBuffer(Buffer, usize);

impl EntityRenderCommand for DrawChunk {
  type Param = SQuery<Read<MeshBuffer>>;

  fn render<'w>(
    _view: Entity,
    item: Entity,
    param: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let MeshBuffer(buf, verticies) = param.get_inner(item).unwrap();
    pass.set_vertex_buffer(0, buf.slice(..));
    pass.draw(0..*verticies as u32 * 6, 0..1 as u32);
    RenderCommandResult::Success
  }
}

#[derive(Default)]
pub struct SelectionBindGroup {
  bind_group: Option<BindGroup>,
}

pub struct SetSelectionBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetSelectionBindGroup<I> {
  type Param = SRes<SelectionBindGroup>;

  fn render<'w>(
    _view: Entity,
    _item: &P,
    bind_group: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    pass.set_bind_group(I, &bind_group.into_inner().bind_group.as_ref().unwrap(), &[]);
    RenderCommandResult::Success
  }
}

#[derive(Default)]
pub struct LightTextureBindGroup {
  bind_group: Option<BindGroup>,
}

pub struct SetLightTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetLightTextureBindGroup<I> {
  type Param = SRes<LightTextureBindGroup>;

  fn render<'w>(
    _view: Entity,
    _item: Entity,
    light_texture_bind_group: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    if let Some(light_texture_bind_group) = light_texture_bind_group.into_inner().bind_group.as_ref() {
      pass.set_bind_group(I, light_texture_bind_group, &[]);
      RenderCommandResult::Success
    } else {
      RenderCommandResult::Failure
    }
  }
}

pub struct ExtractedBlocks {
  blocks: HashMap<DD, BufferVec<SingleSide>>,
}

impl Default for ExtractedBlocks {
  fn default() -> Self {
    Self { blocks: HashMap::new() }
  }
}

impl Plugin for VoxelRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let voxel_shader_vertex = Shader::from_spirv(include_bytes!("../../../assets/shader/voxel.vert.spv").as_slice());
    let voxel_shader_fragment = Shader::from_spirv(include_bytes!("../../../assets/shader/voxel.frag.spv").as_slice());
    shaders.set_untracked(VOXEL_SHADER_VERTEX_HANDLE, voxel_shader_vertex);
    shaders.set_untracked(VOXEL_SHADER_FRAGMENT_HANDLE, voxel_shader_fragment);

    app.add_event::<RemeshEvent>();
    app.add_event::<RelightEvent>();

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app
      .init_resource::<ExtractedBlocks>()
      .init_resource::<VoxelPipeline>()
      .init_resource::<VoxelViewBindGroup>()
      .init_resource::<SelectionBindGroup>()
      .init_resource::<SpecializedRenderPipelines<VoxelPipeline>>()
      .init_resource::<TextureHandle>()
      .init_resource::<LightTextureHandle>()
      .init_resource::<TextureBindGroup>()
      .init_resource::<LightTextureBindGroup>()
      .init_resource::<LightBindGroup>()
      .add_system_to_stage(RenderStage::Extract, extract_chunks)
      .add_system_to_stage(RenderStage::Queue, queue_chunks)
      .add_render_command::<Opaque3d, DrawChunkFull>();
  }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
  pos: [f32; 3],
  uv: [f32; 2],
}

const VERTEX: [[Vertex; 6]; 6] = [
  [
    Vertex {
      pos: [1.0, 0.0, 0.0],
      uv: [0.0, 1.0],
    },
    Vertex {
      pos: [1.0, 0.0, 1.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [1.0, 1.0, 0.0],
      uv: [0.0, 0.0],
    },
    Vertex {
      pos: [1.0, 0.0, 1.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [1.0, 1.0, 1.0],
      uv: [1.0, 0.0],
    },
    Vertex {
      pos: [1.0, 1.0, 0.0],
      uv: [0.0, 0.0],
    },
  ],
  [
    Vertex {
      pos: [0.0, 0.0, 1.0],
      uv: [0.0, 1.0],
    },
    Vertex {
      pos: [0.0, 0.0, 0.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [0.0, 1.0, 1.0],
      uv: [0.0, 0.0],
    },
    Vertex {
      pos: [0.0, 0.0, 0.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [0.0, 1.0, 0.0],
      uv: [1.0, 0.0],
    },
    Vertex {
      pos: [0.0, 1.0, 1.0],
      uv: [0.0, 0.0],
    },
  ],
  [
    Vertex {
      pos: [1.0, 0.0, 1.0],
      uv: [0.0, 1.0],
    },
    Vertex {
      pos: [0.0, 0.0, 1.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [1.0, 1.0, 1.0],
      uv: [0.0, 0.0],
    },
    Vertex {
      pos: [0.0, 0.0, 1.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [0.0, 1.0, 1.0],
      uv: [1.0, 0.0],
    },
    Vertex {
      pos: [1.0, 1.0, 1.0],
      uv: [0.0, 0.0],
    },
  ],
  [
    Vertex {
      pos: [0.0, 0.0, 0.0],
      uv: [0.0, 1.0],
    },
    Vertex {
      pos: [1.0, 0.0, 0.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [0.0, 1.0, 0.0],
      uv: [0.0, 0.0],
    },
    Vertex {
      pos: [1.0, 0.0, 0.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [1.0, 1.0, 0.0],
      uv: [1.0, 0.0],
    },
    Vertex {
      pos: [0.0, 1.0, 0.0],
      uv: [0.0, 0.0],
    },
  ],
  [
    Vertex {
      pos: [0.0, 1.0, 0.0],
      uv: [0.0, 1.0],
    },
    Vertex {
      pos: [1.0, 1.0, 0.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [0.0, 1.0, 1.0],
      uv: [0.0, 0.0],
    },
    Vertex {
      pos: [1.0, 1.0, 0.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [1.0, 1.0, 1.0],
      uv: [1.0, 0.0],
    },
    Vertex {
      pos: [0.0, 1.0, 1.0],
      uv: [0.0, 0.0],
    },
  ],
  [
    Vertex {
      pos: [0.0, 0.0, 1.0],
      uv: [0.0, 1.0],
    },
    Vertex {
      pos: [1.0, 0.0, 1.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [0.0, 0.0, 0.0],
      uv: [0.0, 0.0],
    },
    Vertex {
      pos: [1.0, 0.0, 1.0],
      uv: [1.0, 1.0],
    },
    Vertex {
      pos: [1.0, 0.0, 0.0],
      uv: [1.0, 0.0],
    },
    Vertex {
      pos: [0.0, 0.0, 0.0],
      uv: [0.0, 0.0],
    },
  ],
];
