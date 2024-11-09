use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector3};
use hatchery::util::{
    buffer::{AbstractBuffer, SharedBuffer},
    point_cloud::RenderPoint,
    ConstructionContext,
};
use vulkano::{buffer::BufferUsage, impl_vertex};

pub mod energy;
pub mod verlet;

pub struct Particle {
    position: Point3<f32>,
    velocity: Vector3<f32>,
    mass: f32,
}

impl Particle {
    pub fn new(position: Point3<f32>, velocity: Vector3<f32>, mass: f32) -> Self {
        Self {
            position,
            velocity,
            mass,
        }
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct ParticlePositionMass {
    pos_mass: [f32; 4],
}

impl_vertex!(ParticlePositionMass, pos_mass);

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct ParticleVelocity {
    vel: [f32; 4],
}

impl_vertex!(ParticleVelocity, vel);

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct ParticleAcceleration {
    acc: [f32; 4],
}

impl_vertex!(ParticleAcceleration, acc);

pub struct SimulationBuffers {
    pub points: SharedBuffer<RenderPoint>,
    pub position_mass: SharedBuffer<ParticlePositionMass>,
    pub velocity: SharedBuffer<ParticleVelocity>,
    pub acceleration: SharedBuffer<ParticleAcceleration>,
    pub num_particles: u32,
}

impl SimulationBuffers {
    pub fn new(context: &ConstructionContext, particles: Vec<Particle>) -> Arc<Self> {
        Arc::new(Self {
            points: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| RenderPoint {
                        point_pos: [p.position.x, p.position.y, p.position.z, 0.0],
                    })
                    .collect(),
            ),
            position_mass: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticlePositionMass {
                        pos_mass: [p.position.x, p.position.y, p.position.z, p.mass],
                    })
                    .collect(),
            ),
            velocity: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticleVelocity {
                        vel: [p.velocity.x, p.velocity.y, p.velocity.z, 0.0],
                    })
                    .collect(),
            ),
            acceleration: SharedBuffer::from_vec(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                particles
                    .iter()
                    .map(|p| ParticleAcceleration {
                        acc: [0.0, 0.0, 0.0, 0.0],
                    })
                    .collect(),
            ),
            num_particles: particles.len() as u32,
        })
    }
}
