#version 450

// Triangle covering full-screen square clip space, as per: https://web.archive.org/web/20140719063725/http://www.altdev.co/2011/08/08/interesting-vertex-shader-trick/

void main() {
  uint id = gl_VertexIndex;
  gl_Position = vec4(vec2((id << 1) & 2u, id & 2u) * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
}
