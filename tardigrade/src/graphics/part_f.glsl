#version 450

layout(location = 0) in vec2 f_uv;
layout(location = 1) in float f_brightness;
layout(location = 2) in vec3 f_particle_color;

layout(location = 0) out vec4 f_color;


void main() {
    float radius = clamp(0.5 - length(f_uv - vec2(0.5)), 0.0, 0.5);
    f_color = vec4(f_particle_color, f_brightness * radius * radius);
}
