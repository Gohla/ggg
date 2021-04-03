#version 450

layout(location = 0) in vec2 inPos;
layout(location = 1) in vec2 inTex;

out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec2 outTex;

void main() {
  gl_Position = vec4(inPos, 0.0, 1.0);
  outTex = inTex;
}
