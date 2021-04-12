#version 450

#include "common.glsl"

#define MAX_RECURSION 10
#define NUM_SAMPLES 4

in vec4 gl_FragCoord;
layout(set = 0, binding = 0) uniform Uniform {
  vec4 resolution_and_elapsed;
  vec4 camera_origin_and_vfov;
};

layout(location = 0) out vec4 color;

bool hit_world(Ray r, float t_min, float t_max, out HitRecord rec) {
  rec.t = t_max;
  bool hit = false;
  Material ground = diffuse_material(vec3(0.8, 0.8, 0.0));
  Material center = diffuse_material(vec3(0.7, 0.3, 0.3));
  Material left = dielectric_material(1.5);
  Material right = metal_material(vec3(0.8, 0.6, 0.2), 0.75);
  hit = hit_sphere(sphere(vec3(0.0, -100.5, -1.0), 100.0, ground), r, t_min, rec.t, rec) || hit;
  hit = hit_sphere(sphere(vec3(0.0, 0.0, -1.0), 0.5, center), r, t_min, rec.t, rec) || hit;
  hit = hit_sphere(sphere(vec3(-1.0, 0.0, -1.0), 0.5, left), r, t_min, rec.t, rec) || hit;
  hit = hit_sphere(sphere(vec3(-1.0, 0.0, -1.0), -0.4, left), r, t_min, rec.t, rec) || hit;
  hit = hit_sphere(sphere(vec3(1.0, 0.0, -1.0), 0.5, right), r, t_min, rec.t, rec) || hit;
  return hit;
}

vec3 ray_color(Ray r, inout float seed) {
  HitRecord rec;
  vec3 col = vec3(1.0);
  for (int i = 0; i < MAX_RECURSION; ++i) {
    if (hit_world(r, 0.001, infinity, rec)) {
      Ray scattered;
      vec3 attenuation;
      if (scatter(r, rec, attenuation, scattered, seed)) {
        // Attenuate (absorb) some of the light
        col *= attenuation;
        // Next ray: the scattered ray
        r = scattered;
      } else {
        // All light was attenuated (absorbed).
        return vec3(0.0);
      }
    } else {
      // No hit, use the sky color.
      vec3 unit_direction = normalize(r.direction);
      float t = 0.5 * (unit_direction.y + 1.0);
      col *= (1.0 - t) * vec3(1.0, 1.0, 1.0) + t * vec3(0.5, 0.7, 1.0);
      // Break out of loop early, no more scattering possible.
      return col;
    }
  }
  return col;
}

void main() {
  Camera cam = camera(camera_origin_and_vfov.xyz, vec3(0, 0, -1), vec3(0, 1, 0), camera_origin_and_vfov.w, resolution_and_elapsed.x / resolution_and_elapsed.y);

  // Flip y so that Y goes from top to bottom, as this differs from the RTIOW/OpenGL coordinate systems.
  vec2 uv = vec2(gl_FragCoord.x, resolution_and_elapsed.y - gl_FragCoord.y);

  // Initialise seed.
  float seed = float(base_hash(floatBitsToUint(uv)))/float(0xffffffffU)+resolution_and_elapsed.z;

  // Anti aliasing with box filter, from: http://roar11.com/2019/10/gpu-ray-tracing-in-an-afternoon/
  vec2 rcpRes = vec2(1.0) / resolution_and_elapsed.xy;
  vec3 col = vec3(0.0);
  float rcpNumSamples = 1.0 / float(NUM_SAMPLES);
  for (int x = 0; x < NUM_SAMPLES; ++x)  {
    for (int y = 0; y < NUM_SAMPLES; ++y)    {
      vec2 adj = vec2(float(x), float(y));
      vec2 uv = (uv + adj * rcpNumSamples) * rcpRes;
      col += ray_color(get_ray(cam, uv.x, uv.y), seed);
    }
  }
  col /= float(NUM_SAMPLES * NUM_SAMPLES);

  color = vec4(col, 1.0);
}
