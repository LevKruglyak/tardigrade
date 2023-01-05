use std::sync::Arc;

use hatchery::util::{
    buffer::{AbstractBuffer, DeviceBuffer, SharedBuffer},
    compute::ComputeShader,
    ConstructionContext,
};
use vulkano::{
    buffer::BufferUsage, descriptor_set::WriteDescriptorSet, device::Device,
    shader::ShaderModule,
};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/min.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub struct MinShader {
    data: SharedBuffer<f32>,
    out: SharedBuffer<f32>,
}

impl ComputeShader for MinShader {
    type Constants = ();

    fn push_constants(&self) -> Option<Self::Constants> {
        None
    }

    fn entry_point() -> &'static str {
        "main"
    }

    fn load_module(device: Arc<Device>) -> Arc<ShaderModule> {
        cs::load(device).unwrap()
    }

    fn dispatch_size(&self) -> [u32; 3] {
        [self.data.len() as u32 / 128 + 1, 1, 1]
    }

    fn write_descriptors(&self) -> Vec<WriteDescriptorSet> {
        vec![
            WriteDescriptorSet::buffer(0, self.data.buffer()),
        ]
    }
}

impl MinShader {
    pub fn new(context: &ConstructionContext, particles: Vec<Particle>) -> Self {
        Self {
            position_mass: DeviceBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticlePosition {
                        p_pos: [p.position.x, p.position.y, p.position.z, p.mass],
                    })
                    .collect(),
            ),
            velocity: DeviceBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticleVelocityMass {
                        p_vel_mass: [p.velocity.x, p.velocity.y, p.velocity.z, 0.0],
                    })
                    .collect(),
            ),
            num_particles: particles.len() as u64,
        }
    }

    pub fn particles(&self) -> &DeviceBuffer<ParticlePosition> {
        &self.position_mass
    }

    pub fn velocity_mass(&self) -> &DeviceBuffer<ParticleVelocityMass> {
        &self.velocity
    }
}
