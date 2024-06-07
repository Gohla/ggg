// Taken from:
// 1) https://github.com/hasenbanck/egui_wgpu_backend/blob/master/src/shader/egui.vert
// 2) // 2) https://github.com/emilk/egui/blob/2545939c150379b85517de691da56a46f5ee0d1d/crates/egui-wgpu/src/egui.wgsl

#version 450

layout(std140, set = 0, binding = 0) uniform Uniform {
  vec4 uScreenSize;
};

layout(location = 0) in vec2 inPos;
layout(location = 1) in vec2 inTex;
layout(location = 2) in uint inCol;

out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec2 outTex;
layout(location = 1) out vec4 outCol;

void main() {
  gl_Position = vec4(2.0 * inPos.x / uScreenSize.x - 1.0, 1.0 - 2.0 * inPos.y / uScreenSize.y, 0.0, 1.0);
  outTex = inTex;
  // [u8; 4] SRGB as u32 -> [r, g, b, a]
  outCol = vec4(inCol & 0xFFu, (inCol >> 8) & 0xFFu, (inCol >> 16) & 0xFFu, (inCol >> 24) & 0xFFu) / 255.0;
}
