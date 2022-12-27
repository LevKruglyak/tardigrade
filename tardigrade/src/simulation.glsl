#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer _positions {
    vec3 data[];
} positions;

layout(set = 0, binding = 1) buffer _velocities {
    vec3 data[];
} velocities;

layout(push_constant) uniform SimulationData {
  uint buffer_size;
} sd;

void main() {
    uint buffer_index = gl_GlobalInvocationID.x;

    vec3 position = positions.data[buffer_index];
    vec3 velocity = velocities.data[buffer_index];

    vec3 force = vec3(0.0);

    for (uint i = 0; i < sd.buffer_size; ++i) {
        if (i != buffer_index) {
            vec3 target = positions.data[i];
            float dist2 = dot(position - target, position - target);

            force += 0.00001 * (target - position) / pow(dist2 + 0.01, 1.5);
        }
    }

    velocity += force;
    position += velocity;

    positions.data[buffer_index] = position;
    velocities.data[buffer_index] = velocity;
}
