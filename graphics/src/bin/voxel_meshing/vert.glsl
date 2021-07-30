#version 450

layout(location = 0) in vec3 inPos;

out gl_PerVertex { vec4 gl_Position; };

layout(set = 0, binding = 0)
uniform Uniform {
  mat4 uniViewProj;
};

void main() {
  gl_Position = uniViewProj * vec4(inPos, 1.0);
}
