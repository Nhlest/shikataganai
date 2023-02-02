#version 460

layout(location = 0) in vec2 uv;
layout(location = 1) flat in int cube_selected;
layout(location = 2) flat in int face_selected;
layout(location = 3) in vec3 brightness;
layout(location = 4) in float occlusion;

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 0) uniform texture2DArray t_diffuse;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
  out_color = texture(sampler2DArray(t_diffuse, s_diffuse), vec3(uv, 0));
//  out_color = vec4(occlusion, occlusion, occlusion, 1.0);
  out_color = vec4(occlusion * vec3(out_color.r * brightness.r, out_color.g * brightness.g, out_color.b * brightness.b), out_color.a);
  if (out_color.a <= 0.01) {
    discard;
  } else {
    if (
      cube_selected == 1 &&
      (
        (mod(uv.x * 8.0, 1.0) < 0.02 || mod(uv.x * 8.0, 1.0) > 0.98) ||
        (mod(uv.y * 8.0, 1.0) < 0.02 || mod(uv.y * 8.0, 1.0) > 0.98)
      )
    ) {
      out_color.rgb = vec3(0.0);
    }
    if (
      cube_selected == 1 &&
      face_selected == 1 &&
      (
        (mod(uv.x * 8.0, 1.0) < 0.05 || mod(uv.x * 8.0, 1.0) > 0.95) &&
        (mod(uv.y * 8.0, 1.0) < 0.05 || mod(uv.y * 8.0, 1.0) > 0.95)
      )
    ) {
      out_color.rgb = vec3(0.0);
    }
  }
}
