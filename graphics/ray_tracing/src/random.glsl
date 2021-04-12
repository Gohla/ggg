// Hash functions by Nimitz: https://www.shadertoy.com/view/Xt3cDn

uint base_hash(uvec2 p) {
  p = 1103515245U*((p >> 1U)^(p.yx));
  uint h32 = 1103515245U*((p.x)^(p.y>>3U));
  return h32^(h32 >> 16);
}

float hash1(inout float seed) {
  uint n = base_hash(floatBitsToUint(vec2(seed+=.1, seed+=.1)));
  return float(n)*(1.0/float(0xffffffffU));
}

vec2 hash2(inout float seed) {
  uint n = base_hash(floatBitsToUint(vec2(seed+=.1, seed+=.1)));
  uvec2 rz = uvec2(n, n*48271U);
  return vec2(rz.xy & uvec2(0x7fffffffU))/float(0x7fffffff);
}

vec3 hash3(inout float seed) {
  uint n = base_hash(floatBitsToUint(vec2(seed+=.1, seed+=.1)));
  uvec3 rz = uvec3(n, n*16807U, n*48271U);
  return vec3(rz & uvec3(0x7fffffffU))/float(0x7fffffff);
}


// Random function by Reinder Nijhoff: https://www.shadertoy.com/view/llVcDz

vec3 random_in_unit_sphere(inout float seed) {
  vec3 h = hash3(seed) * vec3(2., 6.28318530718, 1.)-vec3(1, 0, 0);
  float phi = h.y;
  float r = pow(h.z, 1./3.);
  return r * vec3(sqrt(1.-h.x*h.x)*vec2(sin(phi), cos(phi)), h.x);
}

vec2 random_in_unit_disk(inout float seed) {
  vec2 h = hash2(seed) * vec2(1., 6.28318530718);
  float phi = h.y;
  float r = sqrt(h.x);
  return r * vec2(sin(phi), cos(phi));
}


// Adapted random functions from RTIOW.

vec3 random_in_unit_vector(inout float seed) {
  return normalize(random_in_unit_sphere(seed));
}

vec3 random_in_hemisphere(inout float seed, vec3 normal) {
  vec3 in_unit_sphere = random_in_unit_sphere(seed);
  // In the same hemisphere as the normal.
  if (dot(in_unit_sphere, normal) > 0.0) return in_unit_sphere;
  else return -in_unit_sphere;
}
