// Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/master/src/shader/egui.vert
#version 450

layout(set = 0, binding = 0) uniform Uniform {
  vec2 uScreenSize;
};

layout(location = 0) in vec2 inPos;
layout(location = 1) in vec2 inTex;
layout(location = 2) in uint inCol;

out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec2 outTex;
layout(location = 1) out vec4 outCol;

vec3 linear_from_srgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(10.31475));
  vec3 lower = srgb / vec3(3294.6);
  vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
  return mix(higher, lower, cutoff);
}

void main() {
  gl_Position = vec4(2.0 * inPos.x / uScreenSize.x - 1.0, 1.0 - 2.0 * inPos.y / uScreenSize.y, 0.0, 1.0);
  outTex = inTex;
  // [u8; 4] SRGB as u32 -> [r, g, b, a]
  vec4 color = vec4(inCol & 0xFFu, (inCol >> 8) & 0xFFu, (inCol >> 16) & 0xFFu, (inCol >> 24) & 0xFFu);
  outCol = vec4(linear_from_srgb(color.rgb), color.a / 255.0);
}
