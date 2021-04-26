#version 450

#define GRID_LENGTH 8
#define GRID_COUNT 64
#define GRID_COUNT_DIV_4 GRID_COUNT / 4

void main() {
  vec2 uv = tex;
  uv *= GRID_LENGTH;
  uvec2 id = uvec2(uv);
  uv = fract(uv);
  float idx = ud.textureIdxs[id.x/4 + id.y*2][id.x%4];
  outCol = texture(samplerArray, vec3(uv, idx));

  gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
}
