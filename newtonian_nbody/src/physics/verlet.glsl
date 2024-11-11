#version 450

#define PARALLELISM 128

layout(local_size_x = PARALLELISM, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Points { vec4 data[]; } points;
layout(set = 0, binding = 1) buffer PositionMass { vec4 data[]; } pos_mass;
layout(set = 0, binding = 2) buffer Velocity { vec4 data[]; } vel;
layout(set = 0, binding = 3) buffer Acceleration { vec4 data[]; } acc;

layout(push_constant) uniform SimulationData {
    uint buffer_size;
    float dt;
    float G;
    float softening;
} sd;

// #define L 1000.0

#ifdef L
#define POSITION(x) (mod((x), L))
#else
#define POSITION(x) (x)
#endif

vec3 calculate_accel(vec3 target, vec3 position) {
    vec3 diff = target - position;
    float dist2 = dot(diff, diff) + (sd.softening * sd.softening); 
    return sd.G * diff / pow(dist2, 1.5);
}

shared vec4 _pos_mass[PARALLELISM];

void main() {
    uint gi = gl_GlobalInvocationID.x;
    uint li = gl_LocalInvocationID.x;

    if (gi >= sd.buffer_size) {
        return;
    }

    float dt = sd.dt;
    vec3 p = POSITION(pos_mass.data[gi].xyz);
    vec3 v = vel.data[gi].xyz;
    vec3 a = acc.data[gi].xyz;

    // Velocity Verlet Integration
    for (uint q = 0; q < 1; q++) {
        vec3 np = POSITION(p + v * dt + a * (dt * dt * 0.5));
        pos_mass.data[gi].xyz = np;
        barrier();

        // calculate forces
        vec3 na = vec3(0.0, 0.0, 0.0); 
        for (uint i = 0; i < sd.buffer_size; i += PARALLELISM) {
            _pos_mass[li] = pos_mass.data[i + li];
            barrier();

            for (int j = 0; j < PARALLELISM; j++) {
                vec3 op = _pos_mass[j].xyz;
                float m = _pos_mass[j].w;

                na += m * calculate_accel(op, np);
            }

            barrier();
        }

        p = np;
        v = v + (a + na) * (dt * 0.5);;   
        a = na;
    }

    vel.data[gi].xyz = v;
    acc.data[gi].xyz = a;
    points.data[gi].xyz = p;
}
