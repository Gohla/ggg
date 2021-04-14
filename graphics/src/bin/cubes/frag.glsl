#version 450

layout(location = 0) in vec3 inPos;

layout(location = 0) out vec4 outCol;

float random(float x) {
  return fract(sin(x)*1.0);
}

void main() {
  outCol = vec4(random(inPos.x), random(inPos.y), random(inPos.z), 1.0);
}
