#version 450

#include "common.glsl"

#define MAX_RECURSION 10
#define NUM_SAMPLES 4

in vec4 gl_FragCoord;
layout(set = 0, binding = 0) uniform Uniform {
  vec2 resolution;
};

layout(location = 0) out vec4 color;

bool hit_world(Ray r, float t_min, float t_max, out HitRecord rec) {
  rec.t = t_max;
  bool hit = false;
  hit = hit_sphere(sphere(vec3(0.0, 0.0, -1.0), 0.5), r, t_min, rec.t, rec) || hit;
  hit = hit_sphere(sphere(vec3(0.0, -100.5, -1.0), 100.0), r, t_min, rec.t, rec) || hit;
  return hit;
}

vec3 ray_color(Ray r, inout float seed) {
  HitRecord rec;
  vec3 col = vec3(1.0);
  for(int i = 0; i < MAX_RECURSION; ++i) {
    if (hit_world(r, 0.001, infinity, rec)) {
      vec3 target = rec.p + rec.normal + random_in_hemisphere(seed, rec.normal);
      r = ray(rec.p, target - rec.p);
      col *= 0.5;
    } else {
      vec3 unit_direction = normalize(r.direction);
      float t = 0.5 * (unit_direction.y + 1.0);
      col *= (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0);
      return col;
    }
  }
  return col;
}

void main() {
  Camera cam = camera(resolution);

  // Flip y so that Y goes from top to bottom, as this differs from the RTIOW/OpenGL coordinate systems.
  vec2 uv = vec2(gl_FragCoord.x, resolution.y - gl_FragCoord.y);

  float seed = float(base_hash(floatBitsToUint(uv)))/float(0xffffffffU);

  // Anti aliasing with box filter, from: http://roar11.com/2019/10/gpu-ray-tracing-in-an-afternoon/
  vec2 rcpRes = vec2(1.0) / resolution;
  vec3 col = vec3(0.0);
  float rcpNumSamples = 1.0 / float(NUM_SAMPLES);
  for (int x = 0; x < NUM_SAMPLES; ++x)  {
    for (int y = 0; y < NUM_SAMPLES; ++y)    {
      vec2 adj = vec2(float(x), float(y));
      vec2 uv = (uv + adj * rcpNumSamples) * rcpRes;
      col += ray_color(get_ray(cam, uv), seed);
    }
  }
  col /= float(NUM_SAMPLES * NUM_SAMPLES);

  color = vec4(col, 1.0);
}
