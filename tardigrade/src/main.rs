#![allow(unused_variables, dead_code)]

use std::time::Instant;

use cgmath::{Matrix3, Matrix4, Point3, Rad, Vector3};
use egui_implementation::*;
use graphics::renderer::{Renderer, ViewData};
use hatchery::*;
use physics::simulation::{Particle, Simulation};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::UnitBall;

mod graphics;
mod physics;

pub struct TardigradeEngine {
    simulation: Simulation,
    renderer: Renderer,
    elapsed: Instant,
}

fn create_particle(rng: &mut ThreadRng) -> Particle {
    let position = Vector3::from(rng.sample(UnitBall)) * 5.0;
    let velocity = Vector3::new(0.0, 0.0, 0.0);
    let mass = 1.0;

    Particle::new(position, velocity, mass)
}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        println!("using {}", context.api().device_name());

        let num_particles = 100_000;

        let mut rng = thread_rng();
        let simulation = Simulation::new(
            context.api().construction(),
            (0..num_particles)
                .map(|_| create_particle(&mut rng))
                .collect(),
        );

        Self {
            simulation,
            renderer: Renderer::new(context.api().construction(), context.viewport_subpass()),
            elapsed: Instant::now(),
        }
    }

    fn render(&mut self, info: &mut RenderInfo, api: &EngineApi) {
        let elapsed = self.elapsed.elapsed();
        let rotation =
            1.0 * (elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0);
        let rotation = Matrix3::from_angle_y(Rad(0.1 * rotation as f32));

        let aspect_ratio = info.viewport.dimensions[0] / info.viewport.dimensions[1];
        let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.3, 0.3, 1.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
        );
        let scale = Matrix4::from_scale(0.01);

        let view = ViewData {
            world: Matrix4::from(rotation),
            view: (view * scale),
            proj,
        };

        self.renderer
            .draw_particles(self.simulation.particles(), view, info);
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
