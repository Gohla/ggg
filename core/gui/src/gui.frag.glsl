// Taken from: https://github.com/hasenbanck/egui_wgpu_backend/blob/master/src/shader/egui.frag

#version 450

layout(location = 0) in vec2 v_tex_coord;
layout(location = 1) in vec4 v_color;

layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 1) uniform sampler s_texture;
layout(set = 1, binding = 0) uniform texture2D t_texture;

void main() {
  f_color = v_color * texture(sampler2D(t_texture, s_texture), v_tex_coord);
}
