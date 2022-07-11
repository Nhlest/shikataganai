#version 460

layout(set = 0, location = 0) in vec3 position;
layout(set = 0, location = 1) in vec2 uv;

layout(location = 0) out vec2 uv_out;

layout (set = 0, binding = 0) uniform View {
  mat4 view_proj;
  vec3 world_position;
} view;

void main() {
  gl_Position=view.view_proj * vec4(position + vec3(5.0, 37.5, 12.0), 1.0);
  uv_out = uv;
}