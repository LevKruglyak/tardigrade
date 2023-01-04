#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 p_pos;
layout(location = 3) in vec4 p_vel_mass;

layout(location = 0) out vec2 f_uv;
layout(location = 1) out float f_brightness;
layout(location = 2) out vec3 f_particle_color;

layout(push_constant) uniform UniformData {
    mat4 world;
    mat4 view;
    mat4 proj;
    float brightness;
    float size;
} uniforms;

vec3 hue(vec3 inp, float H)
{
  float U = cos(H*3.1415/180.);
  float W = sin(H*3.1415/180.);

  vec3 ret;
  ret.x = (.299+.701*U+.168*W)*inp.x
    + (.587-.587*U+.330*W)*inp.y
    + (.114-.114*U-.497*W)*inp.z;
  ret.y = (.299-.299*U-.328*W)*inp.x
    + (.587+.413*U+.035*W)*inp.y
    + (.114-.114*U+.292*W)*inp.z;
  ret.z = (.299-.3*U+1.25*W)*inp.x
    + (.587-.588*U-1.05*W)*inp.y
    + (.114+.886*U-.203*W)*inp.z;
  return ret;
}

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.size * vec4(position.xy, 0.0, 0.0) + uniforms.proj * worldview * vec4(p_pos.xyz, 1.0);
    f_uv = uv;
    f_brightness = uniforms.brightness;
    f_particle_color = normalize(normalize(p_vel_mass.xyz) + vec3(0.1, 0.0, 0.0));
}
