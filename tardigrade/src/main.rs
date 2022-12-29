#![allow(unused_variables)]

use egui_implementation::*;
use hatchery::{
    util::buffer::{AbstractBuffer, DeviceBuffer},
    *,
};
use vulkano::buffer::BufferUsage;

pub struct TardigradeEngine {}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        println!("using {}", context.api().device_name());

        let buffer: DeviceBuffer<f32> = DeviceBuffer::new(
            context.api().construction(),
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::default()
            },
            1000,
        );

        Self {}
    }

    fn render(&mut self, info: RenderInfo, api: &mut EngineApi) {}

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
