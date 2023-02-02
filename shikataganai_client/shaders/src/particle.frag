#version 460

layout(location = 0) in vec2 uv;
layout(location = 1) in vec3 brightness;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2DArray t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
  out_color = texture(sampler2DArray(t_diffuse, s_diffuse), vec3(uv, 2));
  if (out_color.a < 0.4) {
    discard;
  }
  out_color = vec4(out_color.r * brightness.r, out_color.g * brightness.g, out_color.b * brightness.b, out_color.a);
}
