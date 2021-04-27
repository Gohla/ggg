#version 450

layout(set = 1, binding = 0) uniform texture2D uniform_texture;
layout(set = 1, binding = 1) uniform sampler uniform_sampler;

layout (location = 0) in vec3 in_tex;


layout(location = 0) out vec4 outCol;


void main() {
  outCol = vec4(1.0);
}
