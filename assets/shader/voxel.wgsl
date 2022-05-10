struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]]uv: vec2<f32>;
};

[[stage(vertex)]]
fn vertex(
  [[builtin(vertex_index)]] id: u32,
  [[location(0)]] position: vec3<f32>,
  [[location(1)]] tiles: vec3<u32>,
  [[location(2)]] size: f32,
  [[location(3)]] coord: vec3<f32>,
  [[location(4)]] uv: vec2<f32>
) -> VertexOutput {
    var atlas_size : u32 = u32(8);

    var tile_id : u32 = id / u32(6);

    if (tile_id % u32(2) == u32(0)) {
      tile_id = tiles[tile_id / u32(2)] / u32(65536);
    } else {
      tile_id = tiles[tile_id / u32(2)] % u32(65536);
    }

    var tile_x : u32 = tile_id % atlas_size;
    var tile_y : u32 = tile_id / atlas_size;

    var o: VertexOutput;
    o.position = view.view_proj * (vec4<f32>(coord*size, 1.0) + vec4<f32>(position, 0.0));
    o.uv = (uv + vec2<f32>(f32(tile_x), f32(tile_y))) / f32(atlas_size);
    //o.uv = V[id].uv;
    //var color = color;
    //let r = color % u32(256); color = color / u32(256);
    //let g = color % u32(256); color = color / u32(256);
    //let b = color % u32(256); color = color / u32(256);
    //let a = color % u32(256); color = color / u32(256);
    //o.color = vec4<f32>(f32(r) / 256.0, f32(g) / 256.0, f32(b) / 256.0, f32(a) / 256.0);
    return o;
}

[[group(1), binding(0)]]
var block_texture: texture_2d<f32>;
[[group(1), binding(1)]]
var block_sampler: sampler;

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var color = textureSample(block_texture, block_sampler, in.uv);
    return color;
}
