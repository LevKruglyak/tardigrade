use std::sync::Arc;

use hatchery::util::{
    buffer::{AbstractBuffer, DeviceBuffer},
    compute::ComputeShader,
    ConstructionContext,
};
use vulkano::{
    buffer::BufferUsage, descriptor_set::WriteDescriptorSet, device::Device,
    shader::ShaderModule,
};

use super::{ParticleVelocityMass, ParticlePosition, Particle};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/physics/bh_simulation.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub struct BhSimulationShader {
    position_mass: DeviceBuffer<ParticlePosition>,
    velocity: DeviceBuffer<ParticleVelocityMass>,
    num_particles: u64,
}

impl ComputeShader for BhSimulationShader {
    type Constants = cs::ty::SimulationData;

    fn push_constants(&self) -> Option<Self::Constants> {
        Some(Self::Constants {
            buffer_size: self.num_particles as u32,
            dust_max: self.num_particles as u32,
        })
    }

    fn entry_point() -> &'static str {
        "main"
    }

    fn load_module(device: Arc<Device>) -> Arc<ShaderModule> {
        cs::load(device).unwrap()
    }

    fn dispatch_size(&self) -> [u32; 3] {
        [self.num_particles as u32 / 128 + 1, 1, 1]
    }

    fn write_descriptors(&self) -> Vec<WriteDescriptorSet> {
        vec![
            WriteDescriptorSet::buffer(0, self.position_mass.buffer()),
            WriteDescriptorSet::buffer(1, self.velocity.buffer()),
        ]
    }
}

impl BhSimulationShader {
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
