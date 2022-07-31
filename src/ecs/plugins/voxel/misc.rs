use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::DynamicUniformIndex;
use bevy::render::render_phase::{
  EntityRenderCommand, PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass,
};
use bevy::render::render_resource::ShaderType;
use bevy::render::render_resource::{BindGroup, Buffer, BufferVec};
use bevy::render::view::ViewUniformOffset;
use bevy::utils::hashbrown::HashMap;
use bytemuck_derive::*;
use std::marker::PhantomData;
use std::ops::Deref;

use crate::ecs::resources::block::BlockSprite;
use crate::ecs::resources::chunk_map::BlockAccessorReadOnly;
use crate::util::array::{add_ddd, DD, DDD};

pub enum RelightType {
  // LightSourceAdded,
  // LightSourceRemoved,
  BlockAdded,
  BlockRemoved,
}

#[allow(dead_code)]
pub enum RemeshEvent {
  Remesh(DD),
  Dummy,
}

#[allow(dead_code)]
pub enum RelightEvent {
  Relight(RelightType, DDD),
  Dummy,
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
pub struct SingleSide([SingleVertex; 6]);

fn occluded(neighbours: &BlockAccessorReadOnly, c: DDD, vx: f32, vy: f32, vz: f32, sx: i32, sy: i32, sz: i32) -> u8 {
  let edgex = ((vx * 2.0) - 1.0).round() as i32;
  let edgey = ((vy * 2.0) - 1.0).round() as i32;
  let edgez = ((vz * 2.0) - 1.0).round() as i32;

  let (left, center, right) = if sx != 0 {
    ((sx, edgey, 0), (sx, edgey, edgez), (sx, 0, edgez))
  } else if sy != 0 {
    ((edgex, sy, 0), (edgex, sy, edgez), (0, sy, edgez))
  } else {
    ((edgex, 0, sz), (edgex, edgey, sz), (0, edgey, sz))
  };

  let left = neighbours
    .get_single(add_ddd(left, c))
    .map_or(0, |x| if x.visible() { 1 } else { 0 });
  let center = neighbours
    .get_single(add_ddd(center, c))
    .map_or(0, |x| if x.visible() { 1 } else { 0 });
  let right = neighbours
    .get_single(add_ddd(right, c))
    .map_or(0, |x| if x.visible() { 1 } else { 0 });

  let result = left + center + right;
  if result == 2 && center == 0 {
    3
  } else {
    result
  }
}

impl SingleSide {
  pub fn new(
    (x, y, z): (f32, f32, f32),
    (ix, iy, iz): (i32, i32, i32),
    block: [BlockSprite; 6],
    lighting: (u8, u8),
    neighbours: &BlockAccessorReadOnly,
    ambient_occlusion: bool,
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
    let mut triangles = VERTEX[side];
    let make_face = |triangles: [Vertex; 6]| {
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
          meta: [
            lighting.0,
            lighting.1,
            if ambient_occlusion {
              occluded(
                neighbours,
                (x.round() as i32, y.round() as i32, z.round() as i32),
                vx,
                vy,
                vz,
                ix,
                iy,
                iz,
              )
            } else {
              0
            },
            0,
          ],
        },
      ))
    };
    let single_side = make_face(triangles);
    if single_side.0[0].meta[2] + single_side.0[4].meta[2] > single_side.0[1].meta[2] + single_side.0[2].meta[2] {
      triangles.each_mut().map(|x| {
        if side == 0 || side == 1 {
          x.pos[1] = 1.0 - x.pos[1];
          x.uv[1] = 1.0 - x.uv[1];
        } else if side == 2 || side == 3 {
          x.pos[0] = 1.0 - x.pos[0];
          x.uv[0] = 1.0 - x.uv[0];
        } else {
          x.pos[0] = 1.0 - x.pos[0];
          x.uv[0] = 1.0 - x.uv[0];
        }
      });
      triangles.swap(0, 2);
      triangles.swap(3, 5);
      make_face(triangles)
    } else {
      single_side
    }
  }
}

pub struct TextureHandle(pub Handle<Image>);
pub struct LightTextureHandle(pub Handle<Image>);

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

pub struct ExtractedBlocks {
  pub blocks: HashMap<DD, BufferVec<SingleSide>>,
}

impl Default for ExtractedBlocks {
  fn default() -> Self {
    Self { blocks: HashMap::new() }
  }
}

#[derive(Default, Deref)]
pub struct MeshViewBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default, Deref)]
pub struct VoxelViewBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default, Deref)]
pub struct VoxelTextureBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default, Deref)]
pub struct MeshTextureBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default, Deref)]
pub struct MeshPositionBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default, Deref)]
pub struct SelectionBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default, Deref)]
pub struct LightTextureBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Component)]
pub struct ChunkMeshBuffer(pub Buffer, pub usize);

#[derive(Component)]
pub struct MeshBuffer(pub Buffer, pub Buffer, pub usize);

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
  pos: [f32; 3],
  uv: [f32; 2],
}

pub const VERTEX: [[Vertex; 6]; 6] = [
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

#[derive(Component, ShaderType, Clone)]
pub struct PositionUniform {
  pub transform: Mat4,
}

pub struct SetMeshPositionBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetMeshPositionBindGroup<I> {
  type Param = (
    SRes<MeshPositionBindGroup>,
    SQuery<Read<DynamicUniformIndex<PositionUniform>>>,
  );
  #[inline]
  fn render<'w>(
    _view: Entity,
    item: Entity,
    (mesh_bind_group, mesh_query): SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let mesh_index = mesh_query.get(item).unwrap();
    pass.set_bind_group(
      I,
      &mesh_bind_group.into_inner().as_ref().unwrap(),
      &[mesh_index.index()],
    );
    RenderCommandResult::Success
  }
}

pub struct SetBindGroup<const I: usize, T: Deref<Target = Option<BindGroup>> + Send + Sync + 'static> {
  _phantom: PhantomData<T>,
}
impl<P: PhaseItem, const I: usize, T: Deref<Target = Option<BindGroup>> + Send + Sync + 'static> RenderCommand<P>
  for SetBindGroup<I, T>
{
  type Param = SRes<T>;

  fn render<'w>(
    _view: Entity,
    _item: &P,
    bind_group: SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    if let Some(texture_bind_group) = bind_group.into_inner().deref().as_ref() {
      pass.set_bind_group(I, texture_bind_group, &[]);
      RenderCommandResult::Success
    } else {
      RenderCommandResult::Failure
    }
  }
}

pub struct SetViewBindGroup<const I: usize, T: Deref<Target = Option<BindGroup>> + Send + Sync + 'static> {
  _phantom: PhantomData<T>,
}

impl<P: PhaseItem, const I: usize, T: Deref<Target = Option<BindGroup>> + Send + Sync + 'static> RenderCommand<P>
  for SetViewBindGroup<I, T>
{
  type Param = (SRes<T>, SQuery<Read<ViewUniformOffset>>);

  fn render<'w>(
    view: Entity,
    _item: &P,
    (bind_group, view_query): SystemParamItem<'w, '_, Self::Param>,
    pass: &mut TrackedRenderPass<'w>,
  ) -> RenderCommandResult {
    let view_uniform = view_query.get(view).unwrap();
    if let Some(b) = &bind_group.into_inner().deref().as_ref() {
      pass.set_bind_group(I, b, &[view_uniform.offset]);
      RenderCommandResult::Success
    } else {
      RenderCommandResult::Failure
    }
  }
}
