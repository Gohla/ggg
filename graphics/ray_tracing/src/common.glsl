#include "random.glsl"

// Division by zero creates a value respresenting infinity.

float infinity = 1.0/0.0;


// Ray

struct Ray {
  vec3 origin;
  vec3 direction;
};

Ray ray(vec3 origin, vec3 direction) {
  Ray r;
  r.origin = origin;
  r.direction = direction;
  return r;
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

Camera camera(vec2 resolution, vec3 origin) {
  float image_width = resolution.x;
  float image_height = resolution.y;
  float aspect_ratio = image_width / image_height;

  float viewport_height = 2.0;
  float viewport_width = aspect_ratio * viewport_height;
  float focal_length = 1.0;

  Camera cam;
  cam.origin = origin;
  cam.horizontal = vec3(viewport_width, 0.0, 0.0);
  cam.vertical = vec3(0.0, viewport_height, 0.0);
  cam.lower_left_corner = cam.origin - cam.horizontal / 2.0 - cam.vertical / 2.0 - vec3(0.0, 0.0, focal_length);
  return cam;
}

Ray get_ray(Camera cam, vec2 uv) {
  return ray(cam.origin, cam.lower_left_corner + uv.x * cam.horizontal + uv.y * cam.vertical - cam.origin);
}


  // Materials

  #define MT_DIFFUSE 0
  #define MT_METAL 1
  #define MT_DIELECTRIC 2

struct Material {
  int type;
  vec3 albedo;
  float v;
};

Material diffuse_material(vec3 albedo) {
  Material m;
  m.type = MT_DIFFUSE;
  m.albedo = albedo;
  return m;
}

Material metal_material(vec3 albedo, float roughness) {
  Material m;
  m.type = MT_METAL;
  m.albedo = albedo;
  m.v = roughness;
  return m;
}

Material dielectric_material(float index_of_refraction) {
  Material m;
  m.type = MT_DIELECTRIC;
  m.v = index_of_refraction;
  return m;
}


// Hit

struct HitRecord {
  vec3 p;// Hit point
  vec3 normal;
  Material material;
  float t;// Ray time when hit
  bool front_face;
};

void set_face_normal(inout HitRecord rec, Ray r, vec3 outward_normal) {
  rec.front_face = dot(r.direction, outward_normal) < 0.0;
  rec.normal = rec.front_face ? outward_normal :- outward_normal;
}


// Scatter

vec3 modified_reflect(vec3 v, vec3 n) {
  return v - 2.0*dot(v, n)*n;
}

vec3 modified_refract(vec3 uv, vec3 n, float etai_over_etat) {
  float cos_theta = min(dot(-uv, n), 1.0);
  vec3 r_out_perp =  etai_over_etat * (uv + cos_theta*n);
  vec3 r_out_parallel = -sqrt(abs(1.0 - dot(r_out_perp, r_out_perp))) * n;
  return r_out_perp + r_out_parallel;
}

float reflectance(float cosine, float ref_idx) {
  // Use Schlick's approximation for reflectance.
  float r0 = (1.0-ref_idx) / (1.0+ref_idx);
  r0 = r0*r0;
  return r0 + (1.0-r0)*pow((1.0 - cosine), 5.0);
}

bool scatter(Ray r_in, HitRecord rec, out vec3 attenuation, out Ray scattered, inout float seed) {
  if (rec.material.type == MT_DIFFUSE) {
    vec3 scatter_direction = rec.p + rec.normal + random_in_hemisphere(seed, rec.normal);
    // TODO: Catch degenerate scatter direction ?
    scattered = ray(rec.p, scatter_direction);
    attenuation = rec.material.albedo;
    return true;
  }
  if (rec.material.type == MT_METAL) {
    vec3 reflected = modified_reflect(normalize(r_in.direction), rec.normal);
    scattered = ray(rec.p, reflected + rec.material.v * random_in_unit_sphere(seed));
    attenuation = rec.material.albedo;
    return dot(scattered.direction, rec.normal) > 0.0;
  }
  if (rec.material.type == MT_DIELECTRIC) {
    attenuation = vec3(1.0);
    float index_of_refraction = rec.material.v;
    float refraction_ratio = rec.front_face ? (1.0/index_of_refraction) : index_of_refraction;
    vec3 unit_direction = normalize(r_in.direction);
    float cos_theta = min(dot(-unit_direction, rec.normal), 1.0);
    float sin_theta = sqrt(1.0 - cos_theta*cos_theta);
    bool cannot_refract = refraction_ratio * sin_theta > 1.0;
    vec3 direction;
    if (cannot_refract || reflectance(cos_theta, refraction_ratio) > hash1(seed)) {
      direction = modified_reflect(unit_direction, rec.normal);
    } else {
      direction = modified_refract(unit_direction, rec.normal, refraction_ratio);
    }
    scattered = ray(rec.p, direction);
    return true;
  }
  return false;
}


// Sphere ray tracing

struct Sphere {
  vec3 center;
  float radius;
  Material material;
};

Sphere sphere(vec3 center, float radius, Material material) {
  Sphere s;
  s.center = center;
  s.radius = radius;
  s.material = material;
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
  rec.material = s.material;

  return true;
}
