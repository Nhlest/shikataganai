use crate::ecs::plugins::voxel::{
  extract_meshes, queue_mesh_position_bind_group, queue_meshes, MeshBuffer, MeshTextureBindGroup, MeshViewBindGroup,
  PositionUniform, SetBindGroup, SetMeshPositionBindGroup, SetViewBindGroup,
};
use bevy::asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::core_pipeline::core_3d::Opaque3d;
use bevy::ecs::system::lifetimeless::{Read, SQuery};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::extract_component::UniformComponentPlugin;
use bevy::render::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_phase::{
  AddRenderCommand, EntityRenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
};
use bevy::render::render_resource::ShaderType;
use bevy::render::render_resource::{
  BindGroupLayout, BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState, ColorWrites,
  CompareFunction, DepthStencilState, Face, FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
  RenderPipelineDescriptor, SamplerBindingType, ShaderStages, SpecializedRenderPipeline, SpecializedRenderPipelines,
  TextureFormat, TextureSampleType, TextureViewDimension, VertexBufferLayout, VertexFormat, VertexState,
  VertexStepMode,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::BevyDefault;
use bevy::render::view::ViewUniform;
use bevy::render::{RenderApp, RenderStage};
use bevy::utils::HashMap;
use bevy_rapier3d::prelude::*;
use gltf::buffer::Source;
use gltf::Gltf;
use std::path::Path;
use strum_macros::EnumIter;
use wgpu::{BindGroupLayoutDescriptor, IndexFormat};

#[derive(Default)]
pub struct GltfLoaderII;

struct DataUri<'a> {
  mime_type: &'a str,
  base64: bool,
  data: &'a str,
}

fn split_once(input: &str, delimiter: char) -> Option<(&str, &str)> {
  let mut iter = input.splitn(2, delimiter);
  Some((iter.next()?, iter.next()?))
}

impl<'a> DataUri<'a> {
  fn parse(uri: &'a str) -> Result<DataUri<'a>, ()> {
    let uri = uri.strip_prefix("data:").ok_or(())?;
    let (mime_type, data) = split_once(uri, ',').ok_or(())?;

    let (mime_type, base64) = match mime_type.strip_suffix(";base64") {
      Some(mime_type) => (mime_type, true),
      None => (mime_type, false),
    };

    Ok(DataUri {
      mime_type,
      base64,
      data,
    })
  }

  fn decode(&self) -> Result<Vec<u8>, base64::DecodeError> {
    if self.base64 {
      base64::decode(self.data)
    } else {
      Ok(self.data.as_bytes().to_owned())
    }
  }
}

async fn load_buffers(gltf: &Gltf, load_context: &LoadContext<'_>, asset_path: &Path) -> Option<Vec<Vec<u8>>> {
  const VALID_MIME_TYPES: &[&str] = &["application/octet-stream", "application/gltf-buffer"];

  let mut buffer_data = Vec::new();
  for buffer in gltf.buffers() {
    match buffer.source() {
      Source::Uri(uri) => {
        let uri = percent_encoding::percent_decode_str(uri).decode_utf8().unwrap();
        let uri = uri.as_ref();
        let buffer_bytes = match DataUri::parse(uri) {
          Ok(data_uri) if VALID_MIME_TYPES.contains(&data_uri.mime_type) => data_uri.decode().unwrap(),
          Ok(_) => return None,
          Err(()) => {
            // TODO: Remove this and add dep
            let buffer_path = asset_path.parent().unwrap().join(uri);
            let buffer_bytes = load_context.read_asset_bytes(buffer_path).await.unwrap();
            buffer_bytes
          }
        };
        buffer_data.push(buffer_bytes);
      }
      Source::Bin => {
        if let Some(blob) = gltf.blob.as_deref() {
          buffer_data.push(blob.into());
        } else {
          return None;
        }
      }
    }
  }

  Some(buffer_data)
}

impl AssetLoader for GltfLoaderII {
  fn load<'a>(
    &'a self,
    bytes: &'a [u8],
    load_context: &'a mut LoadContext,
  ) -> BoxedFuture<'a, anyhow::Result<(), anyhow::Error>> {
    Box::pin(async {
      let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
      let gltf = Gltf::from_slice(bytes).unwrap();
      let buffers = load_buffers(&gltf, load_context, load_context.path()).await.unwrap();
      let mesh_map = gltf
        .document
        .meshes()
        .map(|x| (x.name().unwrap(), x))
        .collect::<HashMap<_, _>>();
      let mesh_render = mesh_map.get("Render").unwrap();
      for p in mesh_render.primitives() {
        let reader = p.reader(|r| Some(&buffers[r.index()]));
        let positions = reader.read_positions().unwrap().collect();
        let tex_coords = reader.read_tex_coords(0).unwrap().into_f32().collect();
        let indicies = reader.read_indices().unwrap().into_u32().collect();
        mesh.insert_attribute(
          MeshVertexAttribute::new("POSITIONS", 0, VertexFormat::Float32x3),
          VertexAttributeValues::Float32x3(positions),
        );
        mesh.insert_attribute(
          MeshVertexAttribute::new("TEXCOORD", 1, VertexFormat::Float32x2),
          VertexAttributeValues::Float32x2(tex_coords),
        );
        mesh.set_indices(Some(Indices::U32(indicies)));
      }
      load_context.set_default_asset(LoadedAsset::new(mesh));
      Ok(())
    })
  }

  fn extensions(&self) -> &[&str] {
    &["glb", "gltf"]
  }
}

#[derive(EnumIter, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Meshes {
  TestModel,
}

#[derive(Deref)]
pub struct GltfMeshStorage(pub Option<Handle<Mesh>>);

impl FromWorld for GltfMeshStorage {
  fn from_world(world: &mut World) -> Self {
    let asset_server = world.get_resource::<AssetServer>().unwrap();
    let gltf: Handle<Mesh> = asset_server.load("meshes/meshes.glb");
    GltfMeshStorage(Some(gltf))
  }
}

pub struct MeshRendererPlugin;

pub struct MeshPipeline {
  pub view_layout: BindGroupLayout,
  pub position_layout: BindGroupLayout,
  pub texture_layout: BindGroupLayout,
}

pub const MESH_SHADER_VERTEX_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597699);
pub const MESH_SHADER_FRAGMENT_HANDLE: HandleUntyped =
  HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2763343953151597799);

impl SpecializedRenderPipeline for MeshPipeline {
  type Key = ();

  fn specialize(&self, _key: Self::Key) -> RenderPipelineDescriptor {
    let vertex_formats = vec![VertexFormat::Float32x3, VertexFormat::Float32x2];

    let vertex_layout = VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, vertex_formats);

    RenderPipelineDescriptor {
      vertex: VertexState {
        shader: MESH_SHADER_VERTEX_HANDLE.typed::<Shader>(),
        entry_point: "main".into(),
        shader_defs: vec![],
        buffers: vec![vertex_layout],
      },
      fragment: Some(FragmentState {
        shader: MESH_SHADER_FRAGMENT_HANDLE.typed::<Shader>(),
        shader_defs: vec![],
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
        self.position_layout.clone(),
      ]),
      primitive: PrimitiveState {
        front_face: FrontFace::Cw,
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
      label: Some("mesh_pipeline".into()),
    }
  }
}

impl FromWorld for MeshPipeline {
  fn from_world(world: &mut World) -> Self {
    let render_device = world.resource::<RenderDevice>();
    MeshPipeline {
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
      position_layout: render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[BindGroupLayoutEntry {
          binding: 0,
          visibility: ShaderStages::VERTEX,
          ty: BindingType::Buffer {
            ty: BufferBindingType::Uniform,
            has_dynamic_offset: true,
            min_binding_size: None,
          },
          count: None,
        }],
        label: Some("position_layout"),
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
    }
  }
}

pub type DrawMeshFull = (
  SetItemPipeline,
  SetViewBindGroup<0, MeshViewBindGroup>,
  SetBindGroup<1, MeshTextureBindGroup>,
  SetMeshPositionBindGroup<2>,
  DrawMeshes,
);

pub struct DrawMeshes;
impl EntityRenderCommand for DrawMeshes {
  type Param = SQuery<Read<MeshBuffer>>;

  fn render<'w>(
    _view: Entity,
    item: Entity,
    param: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let MeshBuffer(buf, idx_buffer, indicies) = param.get_inner(item).unwrap();
    pass.set_vertex_buffer(0, buf.slice(..));
    pass.set_index_buffer(idx_buffer.slice(..), 0, IndexFormat::Uint32);
    pass.draw_indexed(0..*indicies as u32, 0, 0..1 as u32);
    RenderCommandResult::Success
  }
}

fn spawn_mesh(
  storage: Res<GltfMeshStorage>,
  mut local: Local<bool>,
  mesh_assets: Res<Assets<Mesh>>,
  mut commands: Commands,
) {
  if !*local {
    if let Some(m) = mesh_assets.get(storage.0.as_ref().unwrap()) {
      commands
        .spawn()
        .insert(storage.0.as_ref().unwrap().clone())
        .insert(Transform::from_xyz(8.0, 39.0, 12.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::from_bevy_mesh(m, &ComputedColliderShape::TriMesh).unwrap())
        .insert(Friction {
          coefficient: 0.0,
          combine_rule: CoefficientCombineRule::Min,
        })
        .insert(SolverGroups::new(0b10, 0b01))
        .insert(CollisionGroups::new(0b10, 0b01))
        .insert(GlobalTransform::default());
      commands
        .spawn()
        .insert(storage.0.as_ref().unwrap().clone())
        .insert(Transform::from_xyz(7.0, 38.0, 12.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::from_bevy_mesh(m, &ComputedColliderShape::TriMesh).unwrap())
        .insert(Friction {
          coefficient: 0.0,
          combine_rule: CoefficientCombineRule::Min,
        })
        .insert(SolverGroups::new(0b10, 0b01))
        .insert(CollisionGroups::new(0b10, 0b01))
        .insert(GlobalTransform::default());
      commands
        .spawn()
        .insert(storage.0.as_ref().unwrap().clone())
        .insert(Transform::from_xyz(6.0, 37.0, 12.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::from_bevy_mesh(m, &ComputedColliderShape::TriMesh).unwrap())
        .insert(Friction {
          coefficient: 0.0,
          combine_rule: CoefficientCombineRule::Min,
        })
        .insert(SolverGroups::new(0b10, 0b01))
        .insert(CollisionGroups::new(0b10, 0b01))
        .insert(GlobalTransform::default());
      commands
        .spawn()
        .insert(storage.0.as_ref().unwrap().clone())
        .insert(Transform::from_xyz(5.0, 36.0, 12.0))
        .insert(RigidBody::Fixed)
        .insert(Collider::from_bevy_mesh(m, &ComputedColliderShape::TriMesh).unwrap())
        .insert(Friction {
          coefficient: 0.0,
          combine_rule: CoefficientCombineRule::Min,
        })
        .insert(SolverGroups::new(0b10, 0b01))
        .insert(CollisionGroups::new(0b10, 0b01))
        .insert(GlobalTransform::default());
      *local = true;
    }
  }
}

impl Plugin for MeshRendererPlugin {
  fn build(&self, app: &mut App) {
    let mut shaders = app.world.resource_mut::<Assets<Shader>>();
    let mesh_shader_vertex = Shader::from_spirv(include_bytes!("../../../../shaders/output/mesh.vert.spv").as_slice());
    let mesh_shader_fragment =
      Shader::from_spirv(include_bytes!("../../../../shaders/output/mesh.frag.spv").as_slice());
    shaders.set_untracked(MESH_SHADER_VERTEX_HANDLE, mesh_shader_vertex);
    shaders.set_untracked(MESH_SHADER_FRAGMENT_HANDLE, mesh_shader_fragment);

    app
      .add_plugin(UniformComponentPlugin::<PositionUniform>::default())
      .init_asset_loader::<GltfLoaderII>()
      .init_resource::<GltfMeshStorage>()
      .add_system(spawn_mesh);

    let render_app = app.get_sub_app_mut(RenderApp).unwrap();
    render_app
      .init_resource::<MeshPipeline>()
      .init_resource::<SpecializedRenderPipelines<MeshPipeline>>()
      .init_resource::<MeshViewBindGroup>()
      .init_resource::<MeshTextureBindGroup>()
      .add_system_to_stage(RenderStage::Extract, extract_meshes)
      .add_system_to_stage(RenderStage::Queue, queue_mesh_position_bind_group)
      .add_system_to_stage(RenderStage::Queue, queue_meshes)
      .add_render_command::<Opaque3d, DrawMeshFull>();
  }
}
