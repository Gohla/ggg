#version 450

layout (location = 0) out vec3 outPos;
out gl_PerVertex { vec4 gl_Position; };

layout(set = 0, binding = 0) uniform Uniform {
  mat4 uniViewProj;
};

struct Instance {
  vec4 pos;
};

layout(std140, set = 0, binding = 1) readonly buffer Instances {
  Instance instances[];
};

void main() {
  uint vx = gl_VertexIndex;
  uint instance = vx >> 3;
  uvec3 xyz = uvec3(vx & 0x1u, (vx & 0x4u) >> 2, (vx & 0x2u) >> 1);
  vec3 uvw = vec3(xyz);
  vec3 pos = uvw * 2.0 - 1.0;

  vec3 instance_pos = instances[instance].pos.xyz;

  gl_Position = uniViewProj * vec4(pos + instance_pos, 1.0);
  outPos = instance_pos;
}
