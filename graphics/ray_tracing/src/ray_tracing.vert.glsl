#version 450

out gl_PerVertex { vec4 gl_Position; };
//layout(location = 0) out vec2 coord;

void main() {
  uint id = gl_VertexIndex;
  //  coord = vec2((id << 1) & 2u, id & 2u);
  //  gl_Position = vec4(coord * vec2(2, -2) + vec2(-1, 1), 0, 1);
  gl_Position = vec4(vec2((id << 1) & 2u, id & 2u) * vec2(2, -2) + vec2(-1, 1), 0, 1);
}
