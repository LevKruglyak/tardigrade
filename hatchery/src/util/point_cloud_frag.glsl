#version 450

layout(location = 0) in vec2 f_uv;
layout(location = 1) in float f_brightness;

layout(location = 0) out vec4 f_color;

#define SIMPLE_PARTICLES

float cubic_spline(float q, float h) {
    if (q < 0.0) {
        q = -q;
    }

    float sig = 0.31830988618 / (h * h * h);
    if (q <= 1.0) {
        return sig * (1.0 - 1.5 * q * q * (1.0 - 0.5 * q));
    } else if (q <= 2.0) {
        return 0.25 * sig * (2.0 - q) * (2.0 - q) * (2.0 - q);
    } else {
        return 0.0;
    }
}

void main() {
#ifdef SIMPLE_PARTICLES
    f_color = vec4(vec3(1.0), length(f_uv - vec2(0.5)) > 0.5 ? 0.0 : f_brightness);
#else
    float radius = clamp(length(f_uv - vec2(0.5)), 0.0, 0.5);
    f_color = vec4(vec3(1.0), 0.1 * cubic_spline(radius * f_brightness, 0.5));
#endif
}
