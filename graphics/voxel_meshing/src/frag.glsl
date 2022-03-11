#version 450
#extension GL_OES_standard_derivatives : enable

layout(location = 0) in vec3 inEyeRelativePosition;

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

  // From: https://stackoverflow.com/a/66206648 and https://www.enkisoftware.com/devlogpost-20150131-1-Normal-generation-in-the-pixel-shader
  vec3 normal = normalize(cross(dFdx(inEyeRelativePosition), dFdy(inEyeRelativePosition)));

  float diffuse = max(dot(normal, lightDirection), 0.0);
  vec3 diffuseColor = light.color * diffuse;

  vec3 viewDirection = normalize(inEyeRelativePosition);
  vec3 halfDirection = normalize(viewDirection + lightDirection);
  float specular = pow(max(dot(normal, halfDirection), 0.0), 32.0);
  vec3 specularColor = specular * light.color;

  vec3 color = (ambientColor + diffuseColor + specularColor) * objectColor;
  outColor = vec4(color, 1.0);
}
