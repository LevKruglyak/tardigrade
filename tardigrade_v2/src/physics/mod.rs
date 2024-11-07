use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use vulkano::impl_vertex;

pub mod verlet;

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
pub struct ParticleVelocityMass {
    part_vel_mass: [f32; 4],
}

impl_vertex!(ParticleVelocityMass, part_vel_mass);
