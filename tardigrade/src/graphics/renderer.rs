use cgmath::Matrix4;
use hatchery::{
    util::{buffer::DeviceBuffer, ConstructionContext},
    RenderInfo, Subpass,
};

use crate::physics::simulation::ParticlePosition;

use super::particles::ParticlesPipeline;

pub struct ViewData {
    pub world: Matrix4<f32>,
    pub view: Matrix4<f32>,
    pub proj: Matrix4<f32>,
}

pub struct Renderer {
    particles_pipeline: ParticlesPipeline,
}

impl Renderer {
    pub fn new(context: &ConstructionContext, subpass: Subpass) -> Self {
        Self {
            particles_pipeline: ParticlesPipeline::new(context, subpass),
        }
    }

    pub fn draw_particles(
        &mut self,
        particles: &DeviceBuffer<ParticlePosition>,
        view: ViewData,
        brightness: f32,
        size: f32,
        info: &mut RenderInfo,
    ) {
        self.particles_pipeline.draw(particles, view, brightness, size, info)
    }
}
