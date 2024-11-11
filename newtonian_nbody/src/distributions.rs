use std::f32::consts::TAU;

use cgmath::{num_traits::Float, InnerSpace, Point3, Quaternion, Rotation, Vector3};
use rand::{rngs::ThreadRng, Rng};
use rand_distr::{Distribution, Uniform, UnitBall};

use crate::{physics::Particle, GRAVITATIONAL_CONSTANT};

#[derive(Debug, Clone, Copy)]
pub struct Plummer {
    scale_length: f64,
    offset: f64,
}

impl Plummer {
    pub fn new(scale_length: f64, offset: f64) -> Self {
        Plummer {
            scale_length,
            offset,
        }
    }
}

impl Distribution<f32> for Plummer {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        let u: f64 = rng.gen();
        let radius = self.scale_length * (u.powf(-2.0 / 3.0) - 1.0).sqrt();
        (radius + self.offset) as f32
    }
}

pub struct BallOfGas {
    mass: f32,
    radius: f32,
    position: Point3<f32>,
    velocity: Vector3<f32>,
}

impl BallOfGas {
    pub fn new(mass: f32, radius: f32, position: Point3<f32>, velocity: Vector3<f32>) -> Self {
        Self {
            mass,
            radius,
            position,
            velocity,
        }
    }

    pub fn get_particles(&self, num_particles: u32, rng: &mut ThreadRng) -> Vec<Particle> {
        let mut particles = Vec::new();

        for _ in 0..num_particles {
            let position = Vector3::from(rng.sample(UnitBall)) * self.radius;

            particles.push(Particle::new(
                self.position + position,
                self.velocity,
                self.mass / num_particles as f32,
            ));
        }

        return particles;
    }
}

pub struct Galaxy<R: Distribution<f32>> {
    black_hole_mass: f32,
    orbit_mass: f32,
    radius: R,
    position: Point3<f32>,
    velocity: Vector3<f32>,
    rotation: Quaternion<f32>,
}

impl<R: Distribution<f32>> Galaxy<R> {
    pub fn new(
        central_mass: f32,
        orbit_mass: f32,
        radius: R,
        position: Point3<f32>,
        velocity: Vector3<f32>,
        look_at: Vector3<f32>,
    ) -> Self {
        Self {
            black_hole_mass: central_mass,
            orbit_mass,
            radius,
            position,
            velocity,
            rotation: Quaternion::from_arc(Vector3::new(1.0, 0.0, 0.0), look_at.normalize(), None),
        }
    }

    pub fn get_particles(&self, num_particles: u32, rng: &mut ThreadRng) -> Vec<Particle> {
        let mut particles = Vec::new();

        let angle_sample = Uniform::new(0.0, TAU);

        for _ in 0..num_particles {
            let radius: f32 = rng.sample(&self.radius);
            let angle: f32 = rng.sample(angle_sample);
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            let position = Vector3::new(0.0, x, y);

            let orbital_speed = (GRAVITATIONAL_CONSTANT * self.black_hole_mass / radius).sqrt();
            let velocity_direction = Vector3::new(0.0, -y, x).normalize();
            let velocity = velocity_direction * orbital_speed;

            particles.push(Particle::new(
                self.position + self.rotation.rotate_vector(position),
                self.velocity + self.rotation.rotate_vector(velocity),
                self.orbit_mass / num_particles as f32,
            ));
        }

        particles.push(Particle::new(
            self.position,
            self.velocity,
            self.black_hole_mass,
        ));

        return particles;
    }
}
