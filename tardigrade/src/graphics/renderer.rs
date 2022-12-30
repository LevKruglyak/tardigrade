use hatchery::{
    util::{buffer::DeviceBuffer, ConstructionContext},
    RenderInfo, Subpass,
};

use crate::physics::simulation::ParticlePosition;

use super::particles::ParticlesPipeline;

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
        info: &mut RenderInfo,
    ) {
        self.particles_pipeline.draw(particles, info)
    }
}
