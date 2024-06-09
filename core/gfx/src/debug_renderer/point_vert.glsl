#version 450

layout(std140, set = 0, binding = 0) uniform Uniform {
  mat4 mvp;
} uni;

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec4 inCol;
layout(location = 2) in float inSize;

layout(location = 0) out vec4 outCol;

void main() {
  outCol = inCol;
  gl_PointSize = inSize;
  gl_Position = uni.mvp * vec4(inPos, 1.0);
}
