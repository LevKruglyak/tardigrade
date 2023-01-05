use egui::plot::{Line, Plot, PlotPoints};
use hatchery::egui_implementation::egui;
use hatchery::egui_implementation::egui::Context;
use hatchery::egui_implementation::EguiImplementation;
use hatchery::*;
use rand::{thread_rng, Rng};

mod min;

pub struct ComputeTesting {
    times: Vec<u64>,
}

impl Engine for ComputeTesting {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let mut times = Vec::new();
        for _ in 0..10 {
            times.push(thread_rng().gen_range(0..=100));
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
