#version 450

#define PARALLELISM 128

layout(local_size_x = PARALLELISM, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer PositionMass {
    vec4 data[];
} pos_mass;

layout(set = 0, binding = 1) buffer Velocity {
    vec4 data[];
} vel;

layout(set = 0, binding = 2) buffer Acceleration {
    vec4 data[];
} acc;

layout(set = 0, binding = 3) buffer EnergyOutput {
    float data[];
} energy;

layout(push_constant) uniform SimulationData {
    uint buffer_size;
} sd;

void main() {
    uint gi = gl_GlobalInvocationID.x;
    uint li = gl_LocalInvocationID.x;

    if (gi >= sd.buffer_size) {
        return;
    }

    vec3 p = pos_mass.data[gi].xyz;
    float m = pos_mass.data[gi].w;
    vec3 v = vel.data[gi].xyz;
    vec3 a = acc.data[gi].xyz;

    energy.data[gi] = 0.5 * m * dot(v, v) - 0.5 * m * dot(a, p);
}
