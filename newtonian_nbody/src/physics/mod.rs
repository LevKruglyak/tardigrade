use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use cgmath::{Point3, Vector3};
use hatchery::util::{
    buffer::{AbstractBuffer, DeviceBuffer, SharedBuffer},
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

type Buffer<T> = DeviceBuffer<T>;

pub struct SimulationBuffers {
    pub points: Buffer<RenderPoint>,
    pub position_mass: Buffer<ParticlePositionMass>,
    pub velocity: Buffer<ParticleVelocity>,
    pub acceleration: Buffer<ParticleAcceleration>,
    pub num_particles: u32,
}

impl SimulationBuffers {
    pub fn new(context: &ConstructionContext, particles: Vec<Particle>) -> Arc<Self> {
        Arc::new(Self {
            points: Buffer::from_iter(
                context,
                BufferUsage {
                    storage_buffer: true,
                    vertex_buffer: true,
                    ..BufferUsage::empty()
                },
                particles.iter().map(|p| RenderPoint {
                    point_pos: [p.position.x, p.position.y, p.position.z, 0.0],
                }),
            ),
            position_mass: Buffer::from_iter(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                particles.iter().map(|p| ParticlePositionMass {
                    pos_mass: [p.position.x, p.position.y, p.position.z, p.mass],
                }),
            ),
            velocity: Buffer::from_iter(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                particles.iter().map(|p| ParticleVelocity {
                    vel: [p.velocity.x, p.velocity.y, p.velocity.z, 0.0],
                }),
            ),
            acceleration: Buffer::from_iter(
                context,
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                particles.iter().map(|p| ParticleAcceleration {
                    acc: [0.0, 0.0, 0.0, 0.0],
                }),
            ),
            num_particles: particles.len() as u32,
        })
    }
}
