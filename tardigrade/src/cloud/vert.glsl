#version 450

layout(location = 0) in vec4 position;

layout(push_constant) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.proj * worldview * vec4(0.2 * position.xyz, 1.0);
}
