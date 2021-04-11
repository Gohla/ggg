#version 450

#include "common.glsl"

in vec4 gl_FragCoord;
layout(set = 0, binding = 0) uniform Uniform {
  vec2 resolution;
};

layout(location = 0) out vec4 color;

bool hit_world(Ray r, float t_min, float t_max, out HitRecord rec) {
  bool hit = false;
  rec.t = t_max;
  if (hit_sphere(sphere(vec3(0.0, 0.0, -1.0), 0.5), r, t_min, rec.t, rec)) hit = true;
  if (hit_sphere(sphere(vec3(0.0, -100.5, -1.0), 100.0), r, t_min, rec.t, rec)) hit = true;
  return hit;
}

vec3 ray_color(Ray r) {
  HitRecord rec;
  float infinity = 1.0/0.0;// Division by zero creates a value respresenting infinity.
  if (hit_world(r, 0.0, infinity, rec)) {
    return 0.5 * (rec.normal + vec3(1.0, 1.0, 1.0));
  }
  vec3 unit_direction = normalize(r.direction);
  float t = 0.5 * (unit_direction.y + 1.0);
  return (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0);
}

void main() {
  Camera cam = camera(resolution);

  // Flip y so that Y goes from top to bottom, as this differs from the RTIOW/OpenGL coordinate systems.
  vec2 uv = vec2(gl_FragCoord.x, resolution.y - gl_FragCoord.y);

  // Anti aliasing with box filter, from: http://roar11.com/2019/10/gpu-ray-tracing-in-an-afternoon/
  vec2 rcpRes = vec2(1.0) / resolution;
  vec3 col = vec3(0.0);
  int numSamples = 4;
  float rcpNumSamples = 1.0 / float(numSamples);
  for (int x = 0; x < numSamples; ++x)  {
    for (int y = 0; y < numSamples; ++y)    {
      vec2 adj = vec2(float(x), float(y));
      vec2 uv = (uv + adj * rcpNumSamples) * rcpRes;
      col += ray_color(get_ray(cam, uv));
    }
  }
  col /= float(numSamples * numSamples);

  color = vec4(col, 1.0);
}
