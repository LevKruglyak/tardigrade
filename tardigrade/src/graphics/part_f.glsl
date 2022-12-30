#version 450

layout(location = 0) in vec2 f_uv;
layout(location = 0) out vec4 f_color;

void main() {
    float radius = 0.25 - length(f_uv - vec2(0.5));
    f_color = vec4(1.0, 1.0, 1.0, 0.1 * radius);
}
