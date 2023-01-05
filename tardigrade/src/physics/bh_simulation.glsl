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

#define SHARED_DATA_SIZE 128 
shared vec4 min_shared_pos_mass[SHARED_DATA_SIZE];
shared vec4 max_shared_pos_mass[SHARED_DATA_SIZE];

void main() {
    uint gi = gl_GlobalInvocationID.x;
    uint li = gl_LocalInvocationID.x;

    if (gi >= sd.buffer_size) {
        return;
    }

    // Step 1: Compute the bounding box
    
    min_shared_pos_mass[li] = position_masses.data[gi];
    max_shared_pos_mass[li] = position_masses.data[gi];

    barrier();

    for (uint s = 128 / 2; s > 0; s >>= 1) {
        if (li < s) {
            min_shared_pos_mass[li] = min(min_shared_pos_mass[li], min_shared_pos_mass[li + s]);
            max_shared_pos_mass[li] = max(max_shared_pos_mass[li], max_shared_pos_mass[li + s]);
        }

        barrier();
    }

    position_masses.data[0].xyz = vec3(0.0);
    velocities.data[0].xyz = vec3(0.0);
}

// __global__ void reduce0(int *g_idata, int *g_odata) {
// extern __shared__ int sdata[];
// // each thread loads one element from global to shared mem
// unsigned int tid = threadIdx.x;
// unsigned int i = blockIdx.x*blockDim.x + threadIdx.x;
// sdata[tid] = g_idata[i];
// __syncthreads();
// // do reduction in shared mem
// for(unsigned int s=1; s < blockDim.x; s *= 2) {
// if (tid % (2*s) == 0) {
// sdata[tid] += sdata[tid + s];
// }
// __syncthreads();
// }
// // write result for this block to global mem
// if (tid == 0) g_odata[blockIdx.x] = sdata[0];
// }
