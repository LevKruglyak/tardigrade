pub mod standard_simulation;
pub mod bh_simulation;

use bytemuck::{Pod, Zeroable};
use cgmath::Vector3;
use vulkano::impl_vertex;

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
