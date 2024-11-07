#version 450

layout(location = 0) in vec2 quad_pos;
layout(location = 1) in vec2 quad_uv;
layout(location = 2) in vec3 point_pos;

layout(location = 0) out vec2 f_uv;
layout(location = 1) out float f_brightness;

layout(push_constant) uniform UniformData {
    mat4 world;
    mat4 view;
    mat4 proj;

    float brightness;
    float size;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.size * vec4(quad_pos, 0.0, 0.0) + uniforms.proj * worldview * vec4(point_pos.xyz, 1.0);
    f_uv = quad_uv;
    f_brightness = uniforms.brightness;
}
