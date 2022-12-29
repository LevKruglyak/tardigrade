#version 450

layout(location = 0) in vec2 f_uv;
layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(f_uv.x, f_uv.y, 0.0, 0.2);
}
