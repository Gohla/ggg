#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;

layout(location = 0) out vec4 outColor;

layout(std140, set = 0, binding = 0) uniform CameraUniform {
  vec4 position;
  mat4 viewProjection;
} camera;

layout(std140, set = 0, binding = 1) uniform LightUniform {
  vec3 color;
  float ambient;
  vec3 direction;
} light;

void main() {
  vec3 objectColor = vec3(1.0, 1.0, 1.0);
  vec3 lightDirection = normalize(light.direction);

  vec3 ambientColor = light.color * light.ambient;

  float diffuse = max(dot(inNormal, lightDirection), 0.0);
  vec3 diffuseColor = light.color * diffuse;

  vec3 viewDirection = normalize(camera.position.xyz - inPosition);
  vec3 halfDirection = normalize(viewDirection + lightDirection);
  float specular = pow(max(dot(inNormal, halfDirection), 0.0), 32.0);
  vec3 specularColor = specular * light.color;

  vec3 color = (ambientColor + diffuseColor + specularColor) * objectColor;
  outColor = vec4(color, 1.0);
}