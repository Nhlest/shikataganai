use crate::ecs::components::blocks::BlockSprite;
use crate::ecs::plugins::rendering::voxel_pipeline::consts::{Vertex, VERTEX};
use bevy::prelude::*;
use bevy::render::render_resource::Buffer;
use bytemuck_derive::{Pod, Zeroable};
use shikataganai_common::ecs::resources::world::GameWorld;
use shikataganai_common::util::array::{add_ddd, DD, DDD};

#[allow(dead_code)]
pub enum RemeshEvent {
  Remesh(DD),
  Dummy,
}

#[derive(Component)]
pub struct ChunkMeshBuffer(pub Buffer, pub usize);

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

fn occluded(neighbours: &GameWorld, c: DDD, vx: f32, vy: f32, vz: f32, sx: i32, sy: i32, sz: i32) -> u8 {
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
    .get(add_ddd(left, c))
    .map_or(0, |x| if x.visible() { 1 } else { 0 });
  let center = neighbours
    .get(add_ddd(center, c))
    .map_or(0, |x| if x.visible() { 1 } else { 0 });
  let right = neighbours
    .get(add_ddd(right, c))
    .map_or(0, |x| if x.visible() { 1 } else { 0 });

  let result = left + center + right;
  if result == 2 && center == 0 {
    3
  } else {
    result
  }
}

pub fn delta_to_side((ix, iy, iz): DDD) -> usize {
  if ix != 0 {
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
  } else if iy == 1 {
    4
  } else {
    5
  }
}

impl SingleSide {
  pub fn new(
    (x, y, z): (f32, f32, f32),
    (ix, iy, iz): (i32, i32, i32),
    block: [BlockSprite; 6],
    lighting: (u8, u8),
    neighbours: &GameWorld,
    ambient_occlusion: bool,
  ) -> Self {
    let fx = x;
    let fy = y;
    let fz = z;
    let side = delta_to_side((ix, iy, iz));
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
