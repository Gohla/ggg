#version 450

layout(set = 0, binding = 0) uniform Uniform {
  mat4 uniViewProj;
};

struct Instance {
  float tex_z[4];
};

layout(std140, set = 0, binding = 1) readonly buffer Instances {
  Instance instances[];
};


out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec3 out_tex;


void main() {
  uint vx = gl_VertexIndex;

  uvec3 xyz = uvec3(vx & 0x1u, (vx & 0x4u) >> 2, 0);
  vec3 uvw = vec3(xyz);
  vec3 pos = uvw * 2.0 - 1.0;
  gl_Position = uniViewProj * vec4(pos, 1.0);

  uint instance_base = vx >> 2; // Divide by 4 to get index into instances array.
  uint instance_sub  = instance_base % 4; // Modulo by 4 of base to get index into tex_z array.
  float tex_z = instances[instance_base].tex_z[instance_sub];
  out_tex = vec3(uvw.xy, tex_z);
}
