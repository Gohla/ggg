#version 450

layout(location = 0) in vec3 inPosition;

out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec3 outPosition;

layout(std140, set = 0, binding = 0) uniform CameraUniform {
  vec4 position;
  mat4 viewProjection;
} camera;

layout(std140, set = 0, binding = 2) uniform ModelUniform {
  mat4 model;
} model;


void main() {
  vec4 pos = model.model * vec4(inPosition, 1.0);
  gl_Position = camera.viewProjection * pos;
  outPosition = vec3(pos);
}
