use bevy::prelude::*;
use bevy::render::render_resource::{BindGroup, Buffer, BufferVec};
use bevy::utils::hashbrown::HashMap;
use bytemuck_derive::*;

use crate::ecs::resources::block::BlockSprite;
use crate::util::array::{DD, DDD};

pub enum RelightType {
  LightSourceAdded,
  LightSourceRemoved,
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

impl SingleSide {
  pub fn new(
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

#[derive(Default)]
pub struct VoxelViewBindGroup {
  pub bind_group: Option<BindGroup>,
}

pub struct ExtractedBlocks {
  pub blocks: HashMap<DD, BufferVec<SingleSide>>,
}

impl Default for ExtractedBlocks {
  fn default() -> Self {
    Self { blocks: HashMap::new() }
  }
}

#[derive(Default)]
pub struct LightBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default)]
pub struct TextureBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default)]
pub struct SelectionBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Default)]
pub struct LightTextureBindGroup {
  pub bind_group: Option<BindGroup>,
}

#[derive(Component)]
pub struct MeshBuffer(pub Buffer, pub usize);

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
