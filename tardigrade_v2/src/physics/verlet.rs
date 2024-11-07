use std::sync::Arc;

use hatchery::util::{
    buffer::{AbstractBuffer, SharedBuffer},
    compute::ComputeShader,
    point_cloud::RenderPoint,
    ConstructionContext,
};
use vulkano::{
    buffer::BufferUsage, descriptor_set::WriteDescriptorSet, device::Device, shader::ShaderModule,
};

use super::{Particle, ParticleVelocityMass};

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/physics/verlet.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub struct Simulation {
    position: SharedBuffer<RenderPoint>,
    velocity_mass: SharedBuffer<ParticleVelocityMass>,
    num_particles: u64,
}

impl Simulation {
    pub fn new(context: &ConstructionContext, particles: Vec<Particle>) -> Self {
        Self {
            position: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| RenderPoint {
                        point_pos: [p.position.x, p.position.y, p.position.z],
                    })
                    .collect(),
            ),
            velocity_mass: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticleVelocityMass {
                        part_vel_mass: [p.velocity.x, p.velocity.y, p.velocity.z, p.mass],
                    })
                    .collect(),
            ),
            num_particles: particles.len() as u64,
        }
    }

    pub fn particles(&self) -> &SharedBuffer<RenderPoint> {
        &self.position
    }

    pub fn velocity_mass(&self) -> &SharedBuffer<ParticleVelocityMass> {
        &self.velocity_mass
    }
}

impl ComputeShader for Simulation {
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
            WriteDescriptorSet::buffer(0, self.position.buffer()),
            WriteDescriptorSet::buffer(1, self.velocity_mass.buffer()),
        ]
    }
}
