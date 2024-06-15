// Taken from:
// 1) https://github.com/hasenbanck/egui_wgpu_backend/blob/master/src/shader/egui.frag
// 2) https://github.com/emilk/egui/blob/2545939c150379b85517de691da56a46f5ee0d1d/crates/egui-wgpu/src/egui.wgsl

#version 450

layout(location = 0) in vec2 inTex;
layout(location = 1) in vec4 inCol;

layout(location = 0) out vec4 outCol;

layout(set = 0, binding = 1) uniform sampler uSampler;
layout(set = 1, binding = 0) uniform texture2D uTexture;

// From: https://github.com/emilk/egui/blob/2545939c150379b85517de691da56a46f5ee0d1d/crates/egui-wgpu/src/egui.wgsl#L18
// 0-1 linear  from  0-1 sRGB gamma
vec3 linear_from_gamma_rgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(0.04045));
  vec3 lower = srgb / vec3(12.92);
  vec3 higher = pow((srgb + vec3(0.055)) / vec3(1.055), vec3(2.4));
  return mix(higher, lower, cutoff);
}

// From: https://github.com/emilk/egui/blob/2545939c150379b85517de691da56a46f5ee0d1d/crates/egui-wgpu/src/egui.wgsl#L26
// 0-1 sRGB gamma  from  0-1 linear
vec3 gamma_from_linear_rgb(vec3 rgb) {
  bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
  vec3 lower = rgb * vec3(12.92);
  vec3 higher = vec3(1.055) * pow(rgb, vec3(1.0 / 2.4)) - vec3(0.055);
  return mix(higher, lower, cutoff);
}

void main() {
  vec4 texLinear = texture(sampler2D(uTexture, uSampler), inTex);
  vec4 texGamma = vec4(gamma_from_linear_rgb(texLinear.rgb), texLinear.a);
  vec4 outColGamma = inCol * texGamma;
  outCol = vec4(linear_from_gamma_rgb(outColGamma.rgb), outColGamma.a);
}
