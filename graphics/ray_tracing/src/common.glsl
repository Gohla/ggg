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
