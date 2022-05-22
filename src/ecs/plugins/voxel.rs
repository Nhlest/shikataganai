use crate::ecs::components::block::BlockId;
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
  BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState,
  ColorWrites, CompareFunction, FragmentState, FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
  RenderPipelineDescriptor, SamplerBindingType, ShaderStages, SpecializedRenderPipeline, SpecializedRenderPipelines,
  TextureFormat, TextureSampleType, TextureViewDimension, VertexBufferLayout, VertexFormat, VertexState,
  VertexStepMode,
};
use bevy::render::render_resource::{Buffer, ShaderType};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::{ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::render::RenderStage;
use bevy::render::{RenderApp, RenderWorld};
use bytemuck_derive::*;
use std::alloc::Layout;
use std::slice;
use wgpu::util::BufferInitDescriptor;
use wgpu::{
  BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindingResource, BufferUsages, DepthStencilState,
  Face,
};

use crate::ecs::components::chunk::Chunk;
use crate::ecs::plugins::camera::Selection;
use crate::ecs::resources::block::BlockSprite;
use crate::ecs::resources::light::{LightMap, Relight, SizedLightMap};

pub struct VoxelRendererPlugin;

pub struct Remesh(pub bool);

impl Remesh {
  pub fn remesh(&mut self) {
    self.0 = true;
  }
}

fn extract_chunks(
  mut render_world: ResMut<RenderWorld>,
  chunks: Query<&Chunk>,
  selection: Res<Option<Selection>>,
  mut remesh: ResMut<Remesh>,
) {
  render_world.insert_resource(selection.clone());
  if !remesh.0 {
    return;
  }
  remesh.0 = false;
  let mut extracted_blocks = render_world.get_resource_mut::<ExtractedBlocks>().unwrap();
  extracted_blocks.blocks.clear();
  for chunk in chunks.iter() {
    chunk.grid.foreach(|(x, y, z), s| {
      if s.block == BlockId::Air {
      } else {
        for (ix, iy, iz) in [(-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0, 0, 1)] {
          if !chunk.grid.in_bounds((x + ix, y + iy, z + iz))
            || chunk.grid[(x + ix, y + iy, z + iz)].block == BlockId::Air
          {
            extracted_blocks
              .blocks
              .push(SingleSide::new((x, y, z), (ix, iy, iz), s.block.into_array_of_faces()));
          }
        }
      }
    });
  }
}

impl SingleSide {
  fn new((x, y, z): (i32, i32, i32), (ix, iy, iz): (i32, i32, i32), block: [BlockSprite; 6]) -> Self {
    let fx = x as f32;
    let fy = y as f32;
    let fz = z as f32;
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
        tile_side: [x, y, z, side as i32],
      },
    ))
  }
}

fn extract_lights(mut render_world: ResMut<RenderWorld>, light: Res<LightMap>) {
  let ((x1, y1, _), (x2, y2, _)) = light.map.bounds;
  let x = x2 - x1 + 1;
  let y = y2 - y1 + 1;
  let size = light.map.size();
  let ptr = unsafe {
    let ptr = std::alloc::alloc(Layout::from_size_align(4 + 4 + size, 4).unwrap());
    let i = ptr as *mut i32;
    i.write(x);
    i.add(1).write(y);
    std::ptr::copy_nonoverlapping(light.map.data(), ptr.add(8), size);
    ptr
  };
  render_world.insert_resource(SizedLightMap {
    ptr: ptr,
    size: size + 8,
  });
}

pub struct VoxelPipeline {
  view_layout: BindGroupLayout,
  texture_layout: BindGroupLayout,
  lights_layout: BindGroupLayout,
  selection_layout: BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct SingleVertex {
  pub position: [f32; 3],
  pub uv: [f32; 2],
  pub tile_side: [i32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SingleSide([SingleVertex; 6]);

pub const VOXEL_SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597899);

impl SpecializedRenderPipeline for VoxelPipeline {
  type Key = ();

  fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
    let shader_defs = Vec::new();
    let vertex_formats = vec![VertexFormat::Float32x3, VertexFormat::Float32x2, VertexFormat::Sint32x4];

    let vertex_layout = VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vertex_formats);

    RenderPipelineDescriptor {
      vertex: VertexState {
        shader: VOXEL_SHADER_HANDLE.typed::<Shader>(),
        entry_point: "vertex".into(),
        shader_defs: shader_defs.clone(),
        buffers: vec![vertex_layout],
      },
      fragment: Some(FragmentState {
        shader: VOXEL_SHADER_HANDLE.typed::<Shader>(),
        shader_defs,
        entry_point: "fragment".into(),
        targets: vec![ColorTargetState {
          format: TextureFormat::bevy_default(),
          blend: Some(BlendState::ALPHA_BLENDING),
          write_mask: ColorWrites::ALL,
        }],
      }),
      layout: Some(vec![
        self.view_layout.clone(),
        self.texture_layout.clone(),
        self.lights_layout.clone(),
        self.selection_layout.clone(),
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
      lights_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::VERTEX,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
          },
          count: None,
        }],
        label: Some("light_view_layout"),
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
    }
  }
}

pub struct TextureHandle(Handle<Image>);

impl FromWorld for TextureHandle {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.resource_mut::<AssetServer>();
    TextureHandle(asset_server.load("texture.png"))
  }
}

fn queue_lights(
  render_device: Res<RenderDevice>,
  light_map: Res<SizedLightMap>,
  mut light_bind_group: ResMut<LightBindGroup>,
  chunk_pipeline: Res<VoxelPipeline>,
) {
  let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
    label: Some("light buffer"),
    contents: light_map.as_slice(),
    usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
  });
  *light_bind_group = LightBindGroup {
    bind_group: Some(render_device.create_bind_group(&BindGroupDescriptor {
      entries: &[BindGroupEntry {
        binding: 0,
        resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
      }],
      label: Some("light_bind_group"),
      layout: &chunk_pipeline.lights_layout,
    })),
  };
}

fn queue_chunks(
  mut commands: Commands,
  extracted_blocks: Res<ExtractedBlocks>,
  mut views: Query<&mut RenderPhase<Opaque3d>>,
  draw_functions: Res<DrawFunctions<Opaque3d>>,
  mut pipelines: ResMut<SpecializedRenderPipelines<VoxelPipeline>>,
  mut pipeline_cache: ResMut<PipelineCache>,
  chunk_pipeline: Res<VoxelPipeline>,
  render_device: Res<RenderDevice>,
  view_uniforms: Res<ViewUniforms>,
  mut voxel_bind_group: ResMut<VoxelViewBindGroup>,
  mut selection_bind_group: ResMut<SelectionBindGroup>,
  gpu_images: Res<RenderAssets<Image>>,
  handle: Res<TextureHandle>,
  mut bind_group: ResMut<TextureBindGroup>,
  selection: Res<Option<Selection>>,
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
    Some(Selection { cube, face }) => [cube[0], cube[1], cube[2], 0, face[0], face[1], face[2], 0],
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

  let draw_chunk_function = draw_functions.read().get_id::<DrawChunkFull>().unwrap();

  let pipeline = pipelines.specialize(&mut pipeline_cache, &chunk_pipeline, ());

  let ptr = extracted_blocks.blocks.as_ptr();
  let s = extracted_blocks.blocks.len() * std::mem::size_of::<SingleSide>();
  let buf = render_device.create_buffer_with_data(&BufferInitDescriptor {
    label: None,
    contents: unsafe { slice::from_raw_parts(ptr as *const u8, s) },
    usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
  });
  let e = commands
    .spawn()
    .insert(MeshBuffer(buf, extracted_blocks.blocks.len()))
    .id();

  for mut view in views.iter_mut() {
    view.add(Opaque3d {
      distance: 0.0,
      draw_function: draw_chunk_function,
      pipeline,
      entity: e,
    });
  }
}

type DrawChunkFull = (
  SetItemPipeline,
  SetVoxelViewBindGroup<0>,
  SetVoxelTextureBindGroup<1>,
  SetLightBindGroup<2>,
  SetSelectionBindGroup<3>,
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

  // fn render<'w>(
  //   _view: Entity,
  //   _item: &P,
  //   param: SystemParamItem<'w, '_, Self::Param>,
  //   pass: &mut TrackedRenderPass<'w>,
  // ) -> RenderCommandResult {
  //   let instances = param.0.blocks.len();
  //   if instances == 0 {
  //     return RenderCommandResult::Success;
  //   }
  //   pass.set_vertex_buffer(0, param.1.into_inner().vertex.buffer().unwrap().slice(..));
  //   pass.set_vertex_buffer(1, param.2.into_inner().vertex.buffer().unwrap().slice(..));
  //   pass.draw(0..36, 0..instances as u32);
  //   RenderCommandResult::Success
  // }
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

#[derive(Default)]
pub struct ExtractedBlocks {
  blocks: Vec<SingleSide>,
}

impl Plugin for VoxelRendererPlugin {
  fn build(&self, app: &mut App) {
    app.insert_resource(Remesh(true));
    app.insert_resource(Relight(true));
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let voxel_shader = Shader::from_wgsl(include_str!("../../../assets/shader/voxel.wgsl"));
    shaders.set_untracked(VOXEL_SHADER_HANDLE, voxel_shader);
    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app
      .init_resource::<ExtractedBlocks>()
      .init_resource::<VoxelPipeline>()
      .init_resource::<VoxelViewBindGroup>()
      .init_resource::<SelectionBindGroup>()
      .init_resource::<SpecializedRenderPipelines<VoxelPipeline>>()
      .init_resource::<TextureHandle>()
      .init_resource::<TextureBindGroup>()
      .init_resource::<LightBindGroup>()
      .add_system_to_stage(RenderStage::Extract, extract_chunks)
      .add_system_to_stage(RenderStage::Extract, extract_lights)
      .add_system_to_stage(RenderStage::Queue, queue_chunks)
      .add_system_to_stage(RenderStage::Queue, queue_lights)
      .add_render_command::<Opaque3d, DrawChunkFull>();
  }
}
