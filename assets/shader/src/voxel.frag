#version 460

layout(location = 0) in vec2 uv;
layout(location = 1) flat in int cube_selected;
layout(location = 2) flat in int face_selected;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2D t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
  out_color = texture(sampler2D(t_diffuse, s_diffuse), uv);
  if (face_selected == 1 && cube_selected == 1) {
    out_color.r += 0.2;
  } else if (cube_selected == 1) {
    out_color.g += 0.2;
  }
}
