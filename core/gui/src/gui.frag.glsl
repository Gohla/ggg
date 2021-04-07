// Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/master/src/shader/egui.frag

#version 450

layout(location = 0) in vec2 inTex;
layout(location = 1) in vec4 inCol;

layout(location = 0) out vec4 outCol;

layout(set = 0, binding = 1) uniform sampler uSampler;
layout(set = 1, binding = 0) uniform texture2D uTexture;

void main() {
  outCol = inCol * texture(sampler2D(uTexture, uSampler), inTex);
}
