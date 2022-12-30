#![allow(unused_variables, dead_code)]

use cgmath::Vector3;
use egui_implementation::*;
use graphics::renderer::Renderer;
use hatchery::*;
use physics::simulation::{Particle, Simulation};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::UnitBall;

mod physics;
mod graphics;

pub struct TardigradeEngine {
    simulation: Simulation,
    renderer: Renderer,
}

fn create_particle(rng: &mut ThreadRng) -> Particle {
    let position = Vector3::from(rng.sample(UnitBall)) * 100.0;
    let velocity = Vector3::new(0.0, 0.0, 0.0);
    let mass = 1.0;

    Particle::new(position, velocity, mass)
}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        println!("using {}", context.api().device_name());

        let num_particles = 10_000;

        let mut rng = thread_rng();
        let simulation = Simulation::new(
            context.api().construction(),
            (0..num_particles)
                .map(|_| create_particle(&mut rng))
                .collect(),
        );

        Self { simulation, renderer: Renderer::new(context.api().construction(), context.viewport_subpass())}
    }

    fn render(&mut self, info: &mut RenderInfo, api: &EngineApi) {
        self.renderer.draw_particles(self.simulation.particles(), info);
    }

    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        egui::SidePanel::left("left_panel")
            .min_width(200.0)
            .resizable(false)
            .show(context, |ui| if ui.button("hello").clicked() {});

        egui::SidePanel::right("right_panel")
            .min_width(200.0)
            .resizable(false)
            .show(context, |ui| {});
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Tardigrade Engine",
            dimensions: LogicalSize::new(1400, 1000),
        },
        features: Features {
            ..Features::empty()
        },
        ..EngineOptions::default()
    };

    EngineLauncher::<TardigradeEngine>::run(options);
}
