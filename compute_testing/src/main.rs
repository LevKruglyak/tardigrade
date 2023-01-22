use std::time::{Duration, Instant};

use egui::plot::{Line, Plot, PlotPoints};
use hatchery::egui_implementation::egui::Context;
use hatchery::egui_implementation::EguiImplementation;
use hatchery::util::compute::ComputeShaderExecutor;
use hatchery::*;
use hatchery::{egui_implementation::egui, util::ConstructionContext};
use min::MinShader;
use rand::{thread_rng, Rng};

mod min;

pub struct ComputeTesting {
    times: Vec<u64>,
}

impl ComputeTesting {
    fn random_min(context: &ConstructionContext, num: u32) -> Duration {
        let mut rng = thread_rng();
        let data: Vec<f32> = (0..num).into_iter().map(|_| 10.0).collect();

        let min_shader = MinShader::new(context, data);
        let executor = ComputeShaderExecutor::new(context, min_shader);

        let start = Instant::now();
        executor.execute(context);
        let dur = start.elapsed();

        println!("minimum: {}", executor.shader().min());
        dur
    }
}

impl Engine for ComputeTesting {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let mut times = Vec::new();
        for _ in 0..10 {
            times.push(
                ComputeTesting::random_min(context.api().construction(), 1023).as_micros() as u64,
            );
        }

        Self { times }
    }

    fn immediate(&mut self, context: &mut Context, api: &mut EngineApi) {
        egui::CentralPanel::default().show(context, |ui| {
            let times: PlotPoints = self
                .times
                .iter()
                .enumerate()
                .map(|(i, x)| [i as f64, *x as f64])
                .collect();
            let line = Line::new(times);
            Plot::new("performance")
                .view_aspect(2.0)
                .show(ui, |plot_ui| plot_ui.line(line));

            ui.label(format!(
                "average time: {} us",
                self.times.iter().sum::<u64>() as f32 / self.times.len() as f32
            ));

            ui.label(format!(
                "subgroup size: {}",
                api.context
                    .device()
                    .physical_device()
                    .properties()
                    .subgroup_size
                    .unwrap_or(0),
            ));

            ui.label(format!(
                "subgroup operations: {:?}",
                api.context
                    .device()
                    .physical_device()
                    .properties()
                    .subgroup_supported_operations
            ));
        });
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions {
            title: "Tardigrade Engine",
            dimensions: LogicalSize::new(400, 400),
        },
        features: Features {
            ..Features::empty()
        },
        ..EngineOptions::default()
    };

    EngineLauncher::<ComputeTesting>::run(options);
}
