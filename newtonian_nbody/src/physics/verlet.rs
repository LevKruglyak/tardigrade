use std::sync::Arc;

use super::{
    Particle, ParticleAcceleration, ParticlePositionMass, ParticleVelocity, SimulationBuffers,
};
use hatchery::util::{
    buffer::{AbstractBuffer, SharedBuffer},
    compute::ComputeShader,
    point_cloud::RenderPoint,
    ConstructionContext,
};
use verlet::ty::SimulationData;
use vulkano::{
    buffer::BufferUsage, descriptor_set::WriteDescriptorSet, device::Device, shader::ShaderModule,
};

hatchery::compute! { "src/physics/verlet.glsl", verlet }

pub struct VerletIntegrator {
    data: Arc<SimulationBuffers>,
    dt: f32,
    g: f32,
    softening: f32,
}

impl VerletIntegrator {
    pub fn new(data: Arc<SimulationBuffers>, dt: f32, g: f32, softening: f32) -> Self {
        Self {
            data,
            g,
            softening,
            dt,
        }
    }
}

impl ComputeShader for VerletIntegrator {
    type Constants = verlet::ty::SimulationData;

    fn push_constants(&self) -> Option<Self::Constants> {
        Some(Self::Constants {
            buffer_size: self.data.num_particles,
            dt: self.dt,
            G: self.g,
            softening: self.softening,
        })
    }

    fn load_module(device: Arc<Device>) -> Arc<ShaderModule> {
        verlet::load(device).unwrap()
    }

    fn dispatch_size(&self) -> [u32; 3] {
        [self.data.num_particles.div_ceil(128), 1, 1]
    }

    fn write_descriptors(&self) -> Vec<WriteDescriptorSet> {
        vec![
            WriteDescriptorSet::buffer(0, self.data.points.buffer()),
            WriteDescriptorSet::buffer(1, self.data.position_mass.buffer()),
            WriteDescriptorSet::buffer(2, self.data.velocity.buffer()),
            WriteDescriptorSet::buffer(3, self.data.acceleration.buffer()),
        ]
    }
}
