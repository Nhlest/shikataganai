#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

layout(set = 3, binding = 0) uniform LightLevel {
  lowp int heaven;
  lowp int hearth;
} light_level;

layout(set = 4, binding = 0) uniform texture2D light_texture;
layout(set = 4, binding = 1) uniform sampler light_sampler;

void main() {
  vec3 brightness = texture(sampler2D(light_texture, light_sampler), vec2(float(light_level.heaven) / 16.0 + 0.5 / 16.0, float(light_level.hearth) / 16.0 + 0.5 / 16.0)).rgb;
  vec4 color = texture(sampler2D(t_diffuse, s_diffuse), uv);
  out_color = vec4(color.rgb * brightness, color.a);
}
