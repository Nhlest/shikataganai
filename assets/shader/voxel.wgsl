struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

struct Light {
  map: array<vec3<f32> >;
};

[[group(2), binding(0)]]
var<storage> light: Light;

struct Selection {
  cube: vec3<i32>;
  face: vec3<i32>;
};

[[group(3), binding(0)]]
var<uniform> selection: Selection;

fn readU8(x: i32, y: i32, z: i32) -> u32 {
//  var offset : u32 = u32(light.x * light.y * z + light.x * y + x);
// 	var ipos : u32 = offset / 4u;
// 	var val_u32 : u32 = light.map[ipos];
// 	var shift : u32 = 8u * (offset % 4u);
// 	var val_u8 : u32 = (val_u32 >> shift) & 0xFFu;

    return 2u;
//	return val_u8;
}

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]]uv: vec2<f32>;
    [[location(1)]]brightness: f32;
    [[location(2)]]cube_selected: i32;
    [[location(3)]]face_selected: i32;
};

[[stage(vertex)]]
fn vertex(
  [[builtin(vertex_index)]] id: u32,
  [[location(0)]] position: vec3<f32>,
  [[location(1)]] uv: vec2<f32>,
  [[location(2)]] tile: vec4<i32>,
  [[location(3)]] metadata: vec4<u8>
) -> VertexOutput {
    var o: VertexOutput;
    o.position = view.view_proj * vec4<f32>(position, 1.0);
    o.uv = uv;

    var side = tile.w;
    var position = vec3<i32>(tile.xyz);

    if (position.x == selection.cube.x && position.y == selection.cube.y && position.z == selection.cube.z) {
      o.cube_selected = 1;
    } else {
      o.cube_selected = 0;
    }

    if (side == 0) {
      position.x = position.x + 1;
    } else if (side == 1) {
      position.x = position.x - 1;
    } else if (side == 2) {
      position.z = position.z + 1;
    } else if (side == 3) {
      position.z = position.z - 1;
    } else if (side == 4) {
      position.y = position.y + 1;
    } else if (side == 5) {
      position.y = position.y - 1;
    }

    if (position.y == selection.face.y && position.y == selection.face.y && position.z == selection.face.z) {
      o.face_selected = 1;
    } else {
      o.face_selected = 0;
    }

    var light_level : u32 = readU8(position.x, position.y, position.z);
    o.brightness = f32(light_level * 16u + 1u) / 256.0;

    return o;
}

[[group(1), binding(0)]]
var block_texture: texture_2d<f32>;
[[group(1), binding(1)]]
var block_sampler: sampler;

[[stage(fragment)]]
fn fragment(vin: VertexOutput) -> [[location(0)]] vec4<f32> {
    var color = textureSample(block_texture, block_sampler, vin.uv);
    color = vec4<f32>(color.rgb * vin.brightness, color.a);
    if (vin.face_selected == 1 && vin.cube_selected == 1) {
      color.r = color.r + 0.2;
    } else if (vin.cube_selected == 1) {
      color.g = color.g + 0.2;
    }
    return color;
}
