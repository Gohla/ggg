#version 450

layout (location = 0) out vec3 outPos;
out gl_PerVertex { vec4 gl_Position; };

layout(std140, set = 0, binding = 0) uniform Uniform {
  vec4 cameraPos;
  mat4 uniViewProj;
};

struct Instance {
  vec4 pos;
};

layout(std430, set = 0, binding = 1) readonly buffer Instances {
  Instance instances[];
};

void main() {
  uint vx = gl_VertexIndex;
  uint instance = vx >> 3;

  vec3 instancePos = instances[instance].pos.xyz;
  vec3 localCameraPos = cameraPos.xyz - instancePos;

  uvec3 xyz = uvec3(vx & 0x1u, (vx & 0x4u) >> 2, (vx & 0x2u) >> 1);

  if (localCameraPos.x > 0) xyz.x = 1 - xyz.x;
  if (localCameraPos.y > 0) xyz.y = 1 - xyz.y;
  if (localCameraPos.z > 0) xyz.z = 1 - xyz.z;

  vec3 uvw = vec3(xyz);
  vec3 pos = uvw * 2.0 - 1.0;

  gl_Position = uniViewProj * vec4(pos + instancePos, 1.0);
  outPos = instancePos;
}
