#version 450

/// Inputs

layout(set = 1, binding = 0) uniform texture2DArray uniform_texture;
layout(set = 1, binding = 1) uniform sampler uniform_sampler;
layout (location = 0) in vec3 in_tex;

/// Outputs

layout(location = 0) out vec4 out_col;

/// Fragment shader

void main() {
  out_col = texture(sampler2DArray(uniform_texture, uniform_sampler), in_tex);
}
