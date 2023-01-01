#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer PositionMass {
    vec4 data[];
} position_masses;

layout(set = 0, binding = 1) buffer Velocity {
    vec4 data[];
} velocities;

layout(push_constant) uniform SimulationData {
  uint buffer_size;
  uint dust_max;
} sd;

vec3 calculate_accel(vec3 target, vec3 position) {
    float dist2 = dot(target - position, target - position);
    return (target - position) / pow(dist2 + 0.1, 1.5);
}

void main() {
    uint buffer_index = gl_GlobalInvocationID.x;

    if (buffer_index < sd.buffer_size) {
        vec3 position = position_masses.data[buffer_index].xyz;
        float mass = position_masses.data[buffer_index].w;

        vec3 velocity = velocities.data[buffer_index].xyz;

        vec3 accel = vec3(0.0);

        for (uint i = 0; i < sd.dust_max; ++i) {
            vec3 target_position = position_masses.data[i].xyz;
            float target_mass = position_masses.data[i].w;
            accel += target_mass * calculate_accel(target_position, position);
        }

        velocity += accel;
        position += velocity;

        position_masses.data[buffer_index] = vec4(position, mass);
        velocities.data[buffer_index].xyz = velocity;
    }
}
