#version 460

layout(location = 0) in vec2 uv;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D block_texture;
layout(set = 1, binding = 1) uniform sampler block_sampler;

void main() {
  out_color = texture(sampler2D(block_texture, block_sampler), uv);
}
