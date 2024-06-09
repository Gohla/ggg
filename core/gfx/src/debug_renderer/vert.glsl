#version 450

layout(std140, set = 0, binding = 0) uniform Uniform {
  mat4 mvp;
} uni;

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec4 inCol;

layout(location = 0) out vec4 outCol;

void main() {
  gl_Position = uni.mvp * vec4(inPos, 1.0);
  outCol = inCol;
}
