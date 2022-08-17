#version 460

layout(set = 0, location = 0) in vec2 position;
layout(set = 0, location = 1) in vec2 uv;
layout(set = 0, location = 2) in float tint;

layout(location = 0) out vec2 uv_out;
layout(location = 1) out float tint_out;

layout (set = 0, binding = 0) uniform View {
  mat4 view_proj;
} view;

void main() {
  gl_Position=view.view_proj * vec4(position.x, position.y, 1.0, 1.0);
  tint_out = tint;
//  gl_Position=vec4(position, 1.0);
  uv_out = uv;
}