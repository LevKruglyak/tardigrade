#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Positions {
    vec4 data[];
} positions;

layout(set = 0, binding = 1) buffer VelocityMasses {
    vec4 data[];
} velocity_masses;

layout(push_constant) uniform SimulationData {
  uint buffer_size;
} sd;

vec3 calculate_accel(vec3 target, vec3 position) {
    float dist2 = dot(position - target, position - target);
    return 0.000001 * (target - position) / pow(dist2 + 0.1, 1.5);
}

void main() {
    uint buffer_index = gl_GlobalInvocationID.x;

    if (buffer_index < sd.buffer_size) {
        vec3 position = positions.data[buffer_index].xyz;
        vec3 velocity = velocity_masses.data[buffer_index].xyz;

        vec3 accel = vec3(0.0);

        for (uint i = 0; i < sd.buffer_size; ++i) {
            float target_mass = velocity_masses.data[i].w;
            accel += target_mass * calculate_accel(positions.data[i].xyz, position);
        }

        velocity += accel;
        position += velocity;

        positions.data[buffer_index] = vec4(position, 0.0);
        velocity_masses.data[buffer_index].xyz = velocity;
    }
}
