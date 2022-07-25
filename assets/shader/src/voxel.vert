#version 460

layout(set = 0, location = 0) in vec3 position;
layout(set = 0, location = 1) in vec2 uv;
layout(set = 0, location = 2) in ivec4 tile;
layout(set = 0, location = 3) in uvec4 meta;

layout(location = 0) out vec2 uv_out;
layout(location = 1) out int cube_selected;
layout(location = 2) out int face_selected;
layout(location = 3) out vec3 brightness;
layout(location = 4) out float occlusion;

layout (set = 2, binding = 0) uniform Selection {
  ivec3 cube;
  ivec3 face;
} selection;

layout (set = 0, binding = 0) uniform View {
  mat4 view_proj;
  vec3 world_position;
} view;

layout(set = 3, binding = 0) uniform texture2D light_texture;
layout(set = 3, binding = 1) uniform sampler light_sampler;

void main() {
  gl_Position=view.view_proj * vec4(position, 1.0);
  ivec3 position = tile.xyz;
  int side = tile.w;
  cube_selected = position == selection.cube ? 1 : 0;

  float brightness_mod = 0.0;
  if (side == 0) {
    position.x++;
    brightness_mod = 0.8;
  } else if (side == 1) {
    position.x--;
    brightness_mod = 0.8;
  } else if (side == 2) {
    position.z++;
    brightness_mod = 0.6;
  } else if (side == 3) {
    position.z--;
    brightness_mod = 0.6;
  } else if (side == 4) {
    position.y++;
    brightness_mod = 1.0;
  } else if (side == 5) {
    position.y--;
    brightness_mod = 0.5;
  }

  face_selected = position == selection.face ? 1 : 0;
  uv_out = uv;

  brightness = brightness_mod * texture(sampler2D(light_texture, light_sampler), vec2(meta[1] / 16.0 + 0.5 / 16.0, meta[0] / 16.0 + 0.5 / 16.0)).rgb;
  occlusion = 1.0 - float(meta[2]) / 4.0;
  occlusion = 1.0 - cos((occlusion * 3.1415926) / 2.0);
}