#version 460

layout(set = 0, location = 0) in vec3 position;
layout(set = 0, location = 1) in vec2 uv;

layout(location = 0) out vec2 uv_out;

layout (set = 0, binding = 0) uniform View {
  mat4 view_proj;
//  vec3 world_position;
} view;

layout (set = 2, binding = 0) uniform Position {
  mat4 position;
  vec3 offset;
} mesh_position;

void main() {
  gl_Position=view.view_proj * mesh_position.position  * vec4(position + mesh_position.offset, 1.0);
  uv_out = uv;
}