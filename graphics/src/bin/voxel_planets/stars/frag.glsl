#version 450

// From:
// - https://twitter.com/turanszkij/status/1519269342290001920
// - https://www.shadertoy.com/view/NtsBzB
// - https://www.shadertoy.com/view/Nlffzj (with colors)
// - https://www.shadertoy.com/view/llj3zV (some camera/UV related code)
// - https://github.com/turanszkij/WickedEngine/commit/d0b3f63511d4ec199dc58b348d0ba11e04925b6b

// 3D Gradient noise from: https://www.shadertoy.com/view/Xsl3Dl
vec3 hash(vec3 p) { // TODO: replace this by something better
  p = vec3(dot(p, vec3(127.1, 311.7, 74.7)),
  dot(p, vec3(269.5, 183.3, 246.1)),
  dot(p, vec3(113.5, 271.9, 124.6)));

  return -1.0 + 2.0*fract(sin(p)*43758.5453123);
}
float noise(in vec3 p) {
  vec3 i = floor(p);
  vec3 f = fract(p);

  vec3 u = f*f*(3.0-2.0*f);

  return mix(mix(mix(dot(hash(i + vec3(0.0, 0.0, 0.0)), f - vec3(0.0, 0.0, 0.0)),
  dot(hash(i + vec3(1.0, 0.0, 0.0)), f - vec3(1.0, 0.0, 0.0)), u.x),
  mix(dot(hash(i + vec3(0.0, 1.0, 0.0)), f - vec3(0.0, 1.0, 0.0)),
  dot(hash(i + vec3(1.0, 1.0, 0.0)), f - vec3(1.0, 1.0, 0.0)), u.x), u.y),
  mix(mix(dot(hash(i + vec3(0.0, 0.0, 1.0)), f - vec3(0.0, 0.0, 1.0)),
  dot(hash(i + vec3(1.0, 0.0, 1.0)), f - vec3(1.0, 0.0, 1.0)), u.x),
  mix(dot(hash(i + vec3(0.0, 1.0, 1.0)), f - vec3(0.0, 1.0, 1.0)),
  dot(hash(i + vec3(1.0, 1.0, 1.0)), f - vec3(1.0, 1.0, 1.0)), u.x), u.y), u.z);
}

// from Unity's black body Shader Graph node
vec3 Unity_Blackbody_float(float Temperature) {
  vec3 color = vec3(255.0, 255.0, 255.0);
  color.x = 56100000. * pow(Temperature, (-3.0 / 2.0)) + 148.0;
  color.y = 100.04 * log(Temperature) - 623.6;
  if (Temperature > 6500.0) color.y = 35200000.0 * pow(Temperature, (-3.0 / 2.0)) + 184.0;
  color.z = 194.18 * log(Temperature) - 1448.6;
  color = clamp(color, 0.0, 255.0)/255.0;
  if (Temperature < 1000.0) color *= Temperature/1000.0;
  return color;
}

layout(std140, set = 0, binding = 0) uniform Uniform {
  vec4 screen_size;
  mat4 view_inverse;

  float stars_threshold;
  float stars_exposure;
  float stars_noise_frequency;
  float temperature_noise_frequency;

  float temperature_minimum;
  float temperature_maximum;
  float temperature_power;
};
layout(location = 0) out vec4 out_col;

void main() {
  // Normalized pixel coordinates (from 0 to 1, or is it -0.5 to 0.5?)
  vec2 uv = gl_FragCoord.xy / screen_size.xy - 0.5;
  uv.x *= screen_size.x / screen_size.y;
  uv.x *= -1.0;// Note: Flip x because for some reason x rotation seems to be inversed?

  // Stars computation:
  vec3 stars_direction = (view_inverse * normalize(vec4(uv, -1.0, 0.0))).xyz;// Note: Also flip z here.
  float stars = pow(clamp(noise(stars_direction * stars_noise_frequency), 0.0f, 1.0f), stars_threshold) * stars_exposure;

  // star color by randomized temperature
  float stars_temperature = noise(stars_direction * temperature_noise_frequency) * 0.5 + 0.5;
  vec3 stars_color = Unity_Blackbody_float(mix(temperature_minimum, temperature_maximum, pow(stars_temperature, temperature_power)));

  // Output to screen
  out_col = vec4(stars_color * stars, 1.0);
}
