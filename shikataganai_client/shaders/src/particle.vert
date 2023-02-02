#version 460

layout(set = 0, location = 0) in vec3 position;
layout(set = 0, location = 1) in uint tile;

layout(location = 0) out vec2 uv;

layout (set = 0, binding = 0) uniform View {
  mat4 view_proj;
  vec3 world_position;
} view;

layout (set = 2, binding = 0) uniform Aspect {
  float aspect_ratio;
} aspect_ratio;

void main() {
  int x = gl_VertexIndex % 2;
  int y = gl_VertexIndex / 2;
  vec4 p1 = view.view_proj * vec4(position, 1.0);
  vec4 p2 = view.view_proj * vec4(position + vec3(0.1, 0.0, 0.0), 1.0);
  float girth = distance(p1, p2);
  gl_Position = p1 + vec4(girth * x - girth / 2, (girth * y - girth / 2) * aspect_ratio.aspect_ratio, 0.0, 0.0);
  uv = vec2(x / 8.0, y / 8.0);
}