#version 450

layout(local_size_x = 1) in;

layout(std430, binding = 0) readonly buffer Input {
    float input_array[];
};

layout(std430, binding = 1) writeonly buffer Output {
    float output_minimum[];
};

layout(push_constant) uniform Constants {
    uint buffer_size;
} constants;

#define MIN float(0xFF800000)

void main() {
    float minimum = MIN;
    if (gl_GlobalInvocationID.x < constants.buffer_size) {
        minimum = input_array[gl_GlobalInvocationID.x];
    }

    minimum = subgroupMin(minimum);

    if (gl_SubgroupInvocationID == 0)
    {
        shared_min[gl_SubgroupID] = minimum;
    }

    memoryBarrierShared();
    barrier();

    if (gl_SubgroupID == 0)
    {
        minimum = gl_SubgroupInvocationID < gl_NumSubgroups ? shared_min[gl_SubgroupInvocationID] : MIN;
        minimum = subgroupMin(minimum);
    }

    if (gl_LocalInvocationID.x == 0)
    {
        output_minimum[gl_WorkGroupID.x] = minimum;
    }
}
