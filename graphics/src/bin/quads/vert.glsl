#version 450

layout(location = 0) in vec2 inPos;
layout(location = 1) in vec2 inTex;
layout(location = 2) in vec4 inModel1;
layout(location = 3) in vec4 inModel2;
layout(location = 4) in vec4 inModel3;
layout(location = 5) in vec4 inModel4;

out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec2 outTex;


layout(std140, set = 1, binding = 0) uniform Uniform {
  mat4 uniViewProj;
};

void main() {
  mat4 model = mat4(inModel1, inModel2, inModel3, inModel4);
  gl_Position = uniViewProj * model * vec4(inPos, 0.0, 1.0);
  outTex = inTex;
}
