#version 450

/// Inputs

layout(set = 0, binding = 0) uniform Uniform {
  mat4 uniViewProj;
};

struct Instance {
  float tex_z[4];
};

//struct Instance {
//  vec2 pos;
//  float rot;
//  float tex_z;
//};

layout(std140, set = 0, binding = 1) readonly buffer Instances {
  Instance instances[];
};

/// Outputs

out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec3 out_tex;

/// Vertex shader

void main() {
  uint vx = gl_VertexIndex;

  uvec2 xy = uvec2(vx & 0x1u, (vx & 0x2u) >> 1);
  vec2 uv = vec2(xy);
  vec2 pos = uv * 2.0 - 1.0;
  gl_Position = uniViewProj * vec4(pos, 1.0, 1.0);

  uint instance_base = vx >> 2; // Divide by 4 to get index into instances array.
  uint instance_sub  = instance_base % 4; // Modulo by 4 of base to get index into tex_z array.
  float tex_z = instances[instance_base].tex_z[instance_sub];
  out_tex = vec3(uv, tex_z);
}
