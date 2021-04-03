#version 450

layout(location = 0) in vec2 inTex;

layout(location = 0) out vec4 outCol;

layout(set = 0, binding = 0) uniform texture2D uTexture;
layout(set = 0, binding = 1) uniform sampler uSampler;

void main() {
  outCol = texture(sampler2D(uTexture, uSampler), inTex);
}
