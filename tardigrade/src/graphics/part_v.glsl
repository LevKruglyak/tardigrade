#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 p_pos;

layout(location = 0) out vec2 f_uv;

layout(push_constant) uniform UniformData {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.proj * worldview * vec4(p_pos.xyz + vec3(position.xy, 0.0), 1.0);
    f_uv = uv;
}
