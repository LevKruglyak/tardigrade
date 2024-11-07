#version 450

layout(local_size_x = 128, local_size_y = 1, local_size_z = 1) in;

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

#define SOFTENING 0.1

vec3 calculate_accel(vec3 target, vec3 position) {
    float dist2 = dot(target - position, target - position);
    return (target - position) / pow(dist2 + SOFTENING, 1.5);
}

#define SHARED_DATA_SIZE 1024 
shared vec4 shared_pos_mass[SHARED_DATA_SIZE];

void main() {
    uint gi = gl_GlobalInvocationID.x;
    uint li = gl_LocalInvocationID.x;

    if (gi >= sd.buffer_size) {
        return;
    }
    
    vec4 pos_mass = position_masses.data[gi];
    vec3 vel = velocities.data[gi].xyz;
    vec3 accel = vec3(0.0);

    for (uint i = 0; i < sd.dust_max; i += SHARED_DATA_SIZE) {
        if (i + li < sd.dust_max) {
            shared_pos_mass[li] = position_masses.data[i + li];
        } else {
            shared_pos_mass[li] = vec4(0.0);
        }

        barrier();

        for (int j = 0; j < gl_WorkGroupSize.x; j++) {
            vec4 other_pos_mass = shared_pos_mass[j];
            accel += other_pos_mass.w * calculate_accel(other_pos_mass.xyz, pos_mass.xyz);
            float dist2 = dot(other_pos_mass.xyz - pos_mass.xyz, other_pos_mass.xyz - pos_mass.xyz);
        }

        barrier();
    }

    vel += accel * 0.01;
    pos_mass.xyz += vel * 0.01;

    position_masses.data[gi] = pos_mass;
    velocities.data[gi].xyz = vel;
}
