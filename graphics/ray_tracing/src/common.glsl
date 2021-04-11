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

// Sphere hit

bool hit_sphere(vec3 center, float radius, Ray r) {
  vec3 oc = r.origin - center;
  float a = dot(r.direction, r.direction);
  float b = 2.0 * dot(oc, r.direction);
  float c = dot(oc, oc) - radius*radius;
  float discriminant = b*b - 4*a*c;
  return (discriminant > 0);
}
