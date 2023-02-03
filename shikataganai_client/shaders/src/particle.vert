#version 460

layout(set = 0, location = 0) in vec3 position;
layout(set = 0, location = 1) in uint tile;
layout(set = 0, location = 2) in uvec2 lighting;

layout(location = 0) out vec2 uv;
layout(location = 1) out vec3 brightness;

layout (set = 0, binding = 0) uniform View {
  mat4 view_proj;
  vec3 world_position;
} view;

layout (set = 2, binding = 0) uniform Aspect {
  float aspect_ratio;
} aspect_ratio;

layout(set = 3, binding = 0) uniform texture2D light_texture;
layout(set = 3, binding = 1) uniform sampler light_sampler;

void main() {
  int x = gl_VertexIndex % 2;
  int y = gl_VertexIndex / 2;
  vec4 p1 = view.view_proj * vec4(position, 1.0);
  vec4 p2 = view.view_proj * vec4(position + vec3(0.1, 0.0, 0.0), 1.0);
  float girth = distance(p1, p2);
  gl_Position = p1 + vec4(girth * x - girth / 2, (girth * y - girth / 2) * aspect_ratio.aspect_ratio, 0.0, 0.0);
  uv = vec2(x / 8.0, y / 8.0);
  brightness = texture(sampler2D(light_texture, light_sampler), vec2(lighting.x / 16.0 + 0.5 / 16.0, lighting.y / 16.0 + 0.5 / 16.0)).rgb;
}