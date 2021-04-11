// Ray

struct Ray {
  vec3 origin;
  vec3 direction;
  float t;
};

Ray ray(vec3 origin, vec3 direction, float t) {
  Ray r;
  r.origin = origin;
  r.direction = direction;
  r.t = t;
  return r;
}

Ray ray(vec3 origin, vec3 direction) {
  return ray(origin, direction, 0.0);
}

vec3 ray_at(Ray ray, float t) {
  return ray.origin + ray.direction * t;
}

// Camera

struct Camera {
  vec3 origin;
  vec3 lower_left_corner;
  vec3 horizontal;
  vec3 vertical;
};

Camera camera(vec2 resolution) {
  float image_width = resolution.x;
  float image_height = resolution.y;
  float aspect_ratio = image_width / image_height;

  float viewport_height = 2.0;
  float viewport_width = aspect_ratio * viewport_height;
  float focal_length = 1.0;

  Camera cam;
  cam.origin = vec3(0.0, 0.0, 0.0);
  cam.horizontal = vec3(viewport_width, 0.0, 0.0);
  cam.vertical = vec3(0.0, viewport_height, 0.0);
  cam.lower_left_corner = cam.origin - cam.horizontal / 2.0 - cam.vertical / 2.0 - vec3(0.0, 0.0, focal_length);
  return cam;
}

Ray get_ray(Camera cam, vec2 uv) {
  return ray(cam.origin, cam.lower_left_corner + uv.x * cam.horizontal + uv.y * cam.vertical - cam.origin);
}


// Hit

struct HitRecord {
  vec3 p;
  vec3 normal;
  float t;
  bool front_face;
};

void set_face_normal(inout HitRecord rec, Ray r, vec3 outward_normal) {
  rec.front_face = dot(r.direction, outward_normal) < 0.0;
  rec.normal = rec.front_face ? outward_normal :- outward_normal;
}


// Sphere ray tracing

struct Sphere {
  vec3 center;
  float radius;
};

Sphere sphere(vec3 center, float radius) {
  Sphere s;
  s.center = center;
  s.radius = radius;
  return s;
}

bool hit_sphere(Sphere s, Ray r, float t_min, float t_max, inout HitRecord rec) {
  vec3 oc = r.origin - s.center;
  float a = dot(r.direction, r.direction);// Length squared = dot product with itself.
  float half_b = dot(oc, r.direction);
  float c = dot(oc, oc) - s.radius * s.radius;

  float discriminant = half_b * half_b - a * c;
  if (discriminant < 0.0) return false;
  float sqrtd = sqrt(discriminant);

  // Find the nearest root that lies in the acceptable range.
  float root = (-half_b - sqrtd) / a;
  if (root < t_min || t_max < root) {
    root = (-half_b + sqrtd) / a;
    if (root < t_min || t_max < root) return false;
  }

  rec.t = root;
  rec.p = ray_at(r, rec.t);
  vec3 outward_normal = (rec.p - s.center) / s.radius;
  set_face_normal(rec, r, outward_normal);

  return true;
}


//
// Hash functions by Nimitz:
// https://www.shadertoy.com/view/Xt3cDn
//

uint base_hash(uvec2 p) {
  p = 1103515245U*((p >> 1U)^(p.yx));
  uint h32 = 1103515245U*((p.x)^(p.y>>3U));
  return h32^(h32 >> 16);
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

//
// Random functions by Reinder Nijhoff:
// https://www.shadertoy.com/view/llVcDz
//

vec3 random_in_unit_sphere(inout float seed) {
  vec3 h = hash3(seed) * vec3(2., 6.28318530718, 1.)-vec3(1, 0, 0);
  float phi = h.y;
  float r = pow(h.z, 1./3.);
  return r * vec3(sqrt(1.-h.x*h.x)*vec2(sin(phi), cos(phi)), h.x);
}

vec3 random_in_unit_vector(inout float seed) {
  return normalize(random_in_unit_sphere(seed));
}

vec3 random_in_hemisphere(inout float seed, vec3 normal) {
  vec3 in_unit_sphere = random_in_unit_sphere(seed);
  if (dot(in_unit_sphere, normal) > 0.0) return in_unit_sphere;// In the same hemisphere as the normal
  else return -in_unit_sphere;
}

// Division by zero creates a value respresenting infinity.
float infinity = 1.0/0.0;
