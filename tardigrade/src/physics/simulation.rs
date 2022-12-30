use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use hatchery::util::{
    buffer::{AbstractBuffer, DeviceBuffer},
    ConstructionContext,
};
use vulkano::{buffer::BufferUsage, impl_vertex};

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
    data: [f32; 4],
}

impl_vertex!(ParticleVelocityMass, data);

pub struct Simulation {
    positions: DeviceBuffer<ParticlePosition>,
    velocity_masses: DeviceBuffer<ParticleVelocityMass>,
    num_particles: u64,
}

impl Simulation {
    pub fn new(context: &ConstructionContext, particles: Vec<Particle>) -> Self {
        Self {
            positions: DeviceBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticlePosition {
                        p_pos: [p.position.x, p.position.y, p.position.z, 0.0],
                    })
                    .collect(),
            ),
            velocity_masses: DeviceBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticleVelocityMass {
                        data: [p.velocity.x, p.velocity.y, p.velocity.z, p.mass],
                    })
                    .collect(),
            ),
            num_particles: particles.len() as u64,
        }
    }

    pub fn particles(&self) -> &DeviceBuffer<ParticlePosition> {
        &self.positions
    }
}
