#version 450

// Outputs
/// Builtin vertex position
out gl_PerVertex { vec4 gl_Position; };

const vec2 positions[3] = vec2[3](
vec2(0.0, 0.5),
vec2(-0.5, -0.5),
vec2(0.5, -0.5)
);

void main() {
  gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
