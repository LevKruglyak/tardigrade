#version 450

layout(location = 0) in vec2 f_uv;
layout(location = 1) in float f_brightness;

layout(location = 0) out vec4 f_color;

// #define SIMPLE_PARTICLES

void main() {
#ifdef SIMPLE_PARTICLES
    f_color = vec4(vec3(1.0), length(f_uv - vec2(0.5)) > 0.5 ? 0.0 : f_brightness);
#else
    float radius = clamp(0.5 - length(f_uv - vec2(0.5)), 0.0, 0.5);
    f_color = vec4(vec3(1.0), f_brightness * radius * radius);
#endif
}
