use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use hatchery::util::{
    buffer::{AbstractBuffer, DeviceBuffer},
    compute::ComputeShader,
    ConstructionContext,
};
use vulkano::{
    buffer::BufferUsage, descriptor_set::WriteDescriptorSet, device::Device, impl_vertex,
    shader::ShaderModule,
};

pub struct Particle {
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    mass: f32,
}

impl Particle {
    pub fn new(position: Vector3<f32>, velocity: Vector3<f32>, mass: f32) -> Self {
        Self {
            position,
            velocity,
            mass,
        }
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct ParticlePosition {
    p_pos: [f32; 4],
}

impl_vertex!(ParticlePosition, p_pos);

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct ParticleVelocityMass {
    p_vel_mass: [f32; 4],
}

impl_vertex!(ParticleVelocityMass, p_vel_mass);

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/physics/simulation.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

pub struct SimulationShader {
    position_mass: DeviceBuffer<ParticlePosition>,
    velocity: DeviceBuffer<ParticleVelocityMass>,
    num_particles: u64,
}

impl ComputeShader for SimulationShader {
    type Constants = cs::ty::SimulationData;

    fn push_constants(&self) -> Option<Self::Constants> {
        Some(Self::Constants {
            buffer_size: self.num_particles as u32,
            dust_max: self.num_particles as u32 / 500,
        })
    }

    fn entry_point() -> &'static str {
        "main"
    }

    fn load_module(device: Arc<Device>) -> Arc<ShaderModule> {
        cs::load(device).unwrap()
    }

    fn dispatch_size(&self) -> [u32; 3] {
        [self.num_particles as u32 / 64 + 1, 1, 1]
    }

    fn write_descriptors(&self) -> Vec<WriteDescriptorSet> {
        vec![
            WriteDescriptorSet::buffer(0, self.position_mass.buffer()),
            WriteDescriptorSet::buffer(1, self.velocity.buffer()),
        ]
    }
}

impl SimulationShader {
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
