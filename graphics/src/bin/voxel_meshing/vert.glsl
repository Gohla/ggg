#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;

out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec3 outPosition;
layout(location = 1) out vec3 outNormal;

layout(set = 0, binding = 0)
uniform CameraUniform {
  vec4 position;
  mat4 viewProjection;
} camera;

void main() {
  gl_Position = camera.viewProjection * vec4(inPosition, 1.0);
  outPosition = inPosition;
  outNormal = inNormal;
}
