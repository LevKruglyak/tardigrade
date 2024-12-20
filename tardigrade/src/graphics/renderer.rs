use hatchery::{
    util::{
        buffer::SharedBuffer,
        ConstructionContext,
    },
    RenderInfo, Subpass,
};

use crate::physics::{ParticlePosition, ParticleVelocityMass};

use super::{particles::ParticlesPipeline, view::ViewData};

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
        particles: &SharedBuffer<ParticlePosition>,
        velocity_mass: &SharedBuffer<ParticleVelocityMass>,
        view: ViewData,
        brightness: f32,
        size: f32,
        info: &mut RenderInfo,
    ) {
        self.particles_pipeline
            .draw(particles, velocity_mass, view, brightness, size, info)
    }
}
