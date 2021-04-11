#version 450

#include "common.glsl"

//layout(location = 0) in vec2 coord;
in vec4 gl_FragCoord;
layout(set = 0, binding = 0) uniform Uniform {
  vec2 resolution;
};

layout(location = 0) out vec4 color;

vec3 ray_color(Ray r) {
  if (hit_sphere(vec3(0.0, 0.0, -1.0), 0.5, r)) {
    return vec3(1.0, 0.0, 0.0);
  }
  vec3 direction_normalized = normalize(r.direction);
  float t = 0.5 * (direction_normalized.y + 1.0);
  return (1.0 - t) * vec3(1.0) + t * vec3(0.5, 0.7, 1.0);
}

void main() {
  float image_width = resolution.x;
  float image_height = resolution.y;
  float aspect_ratio = image_width / image_height;

  float viewport_height = 2.0;
  float viewport_width = aspect_ratio * viewport_height;
  float focal_length = 1.0;

  vec3 origin = vec3(0.0, 0.0, 0.0);
  vec3 horizontal = vec3(viewport_width, 0.0, 0.0);
  vec3 vertical = vec3(0.0, viewport_height, 0.0);
  vec3 lower_left_corner = origin - horizontal / 2.0 - vertical / 2.0 - vec3(0.0, 0.0, focal_length);

  vec2 coord = vec2(gl_FragCoord.x, resolution.y - gl_FragCoord.y);// Flip y so that Y goes from top to bottom.
  float u = coord.x / (image_width - 1);
  float v = coord.y / (image_height - 1);
  Ray r = ray(origin, lower_left_corner + u * horizontal + v * vertical - origin);
  color = vec4(ray_color(r), 1.0);
}
