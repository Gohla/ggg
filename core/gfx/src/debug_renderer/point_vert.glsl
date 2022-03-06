#version 450

layout(std140, set = 0, binding = 0) uniform Uniform {
  mat4 uniViewProj;
};

layout(location = 0) in vec3 inPos;
layout(location = 1) in vec4 inCol;
layout(location = 2) in float inSize;

out gl_PerVertex { vec4 gl_Position; float gl_PointSize; };
layout(location = 0) out vec4 outCol;

void main() {
  gl_Position = uniViewProj * vec4(inPos, 1.0);
  gl_PointSize = inSize;
  outCol = inCol;
}
