#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
  out_color = texture(sampler2D(t_diffuse, s_diffuse), uv);
//  out_color = vec4(1.0, 0.0, 0.0, 1.0);
}
