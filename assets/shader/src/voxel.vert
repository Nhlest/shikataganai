#version 460

layout(set = 0, location = 0) in vec3 position;
layout(set = 0, location = 1) in vec2 uv;
layout(set = 0, location = 2) in ivec4 tile;
layout(set = 0, location = 3) in uvec4 meta;

layout(location = 0) out vec2 uv_out;
layout(location = 1) out int cube_selected;
layout(location = 2) out int face_selected;

layout (set = 2, binding = 0) uniform Selection {
  ivec3 cube;
  ivec3 face;
} selection;

layout (set = 0, binding = 0) uniform View {
  mat4 view_proj;
  vec3 world_position;
} view;

void main() {
  gl_Position=view.view_proj * vec4(position, 1.0);
  ivec3 position = tile.xyz;
  int side = tile.w;
  cube_selected = position == selection.cube ? 1 : 0;

  if (side == 0) {
    position.x++;
  } else if (side == 1) {
    position.x--;
  } else if (side == 2) {
    position.z++;
  } else if (side == 3) {
    position.z--;
  } else if (side == 4) {
    position.y++;
  } else if (side == 5) {
    position.y--;
  }

  face_selected = position == selection.face ? 1 : 0;
  uv_out = uv;
}