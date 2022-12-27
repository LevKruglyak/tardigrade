#![allow(unused_variables)]

use std::time::{Duration, Instant};

use cloud::CloudPipeline;
// use cloud::CloudPipeline;
use gui_implementation::egui;
use simulation::Simulation;
use tardigrade_launcher::*;
use vulkano::device::Features;

mod cloud;
mod simulation;

pub struct TardigradeEngine {
    cloud_pipeline: CloudPipeline,
    simulation: Simulation,
    last_duration: Duration,
}

impl Engine for TardigradeEngine {
    type Gui = gui_implementation::EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        println!("using {}", context.api().device_name());

        let num_particles = 50_000;
        let simulation = Simulation::new(context.api().compute_queue(), num_particles);

        let pipeline = CloudPipeline::new(
            context.api().graphics_queue(),
            context.viewport_subpass(),
            simulation.positions(),
            num_particles,
        );

        Self {
            cloud_pipeline: pipeline,
            simulation,
            last_duration: Duration::default(),
        }
    }

    fn render(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        subpass: Subpass,
        viewport: Viewport,
        api: &mut EngineApi,
    ) {
        self.cloud_pipeline.draw(command_buffer, viewport);
    }

    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        egui::SidePanel::left("left_panel")
            .min_width(200.0)
            .resizable(false)
            .show(context, |ui| {
                ui.label(format!("last time: {}", self.last_duration.as_millis()));
            });

        let start = Instant::now();
        for i in 0..30 {
            self.simulation.advance();
        }
        self.last_duration = start.elapsed();

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
            // wide_lines: true,
            ..Features::none()
        },
        ..EngineOptions::default()
    };

    EngineLauncher::<TardigradeEngine>::run(options);
}
