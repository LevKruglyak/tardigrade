#![allow(unused_variables, unused_imports, dead_code)]

use std::{
    f32::consts::TAU,
    sync::Arc,
    time::{Duration, Instant},
};

use cgmath::{num_traits::Pow, InnerSpace, Point3, Vector3, Zero};
use distributions::{BallOfGas, Galaxy, Plummer};
use egui::{
    plot::{HLine, Line, Plot, PlotPoints},
    Color32, DragValue, Grid, Window,
};
use egui_implementation::*;
use egui_widgets::*;
use hatchery::{
    dpi::PhysicalPosition,
    event::{
        ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    },
    util::compute::ComputeShaderExecutor,
    *,
};
use noise::{core::perlin, NoiseFn, Perlin};
use physics::{energy::EnergyCalculator, verlet::VerletIntegrator, Particle, SimulationBuffers};
use rand::{thread_rng, Rng};
use rand_distr::{Uniform, UnitBall, UnitCircle};
use util::{buffer::AbstractBuffer, camera::Camera, point_cloud::PointCloudPipeline};

mod distributions;
mod physics;

const GRAVITATIONAL_CONSTANT: f32 = 0.01;

pub struct GuiState {
    active: bool,
    brightness: f32,
    scale: f32,

    show_energy: bool,
    last_simulation_time: Duration,
    energy: Vec<f32>,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            active: false,
            brightness: 1.0,
            scale: 0.2,

            last_simulation_time: Duration::default(),
            show_energy: false,
            energy: Vec::new(),
        }
    }
}

pub struct TardigradeEngine {
    simulation: Arc<SimulationBuffers>,
    integrator: ComputeShaderExecutor<VerletIntegrator>,
    energy: ComputeShaderExecutor<EnergyCalculator>,
    render: PointCloudPipeline,
    camera: Camera,
    state: GuiState,
}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let mut particles = Vec::new();
        let num_particles = 300_000;
        let total_mass = 1.0;
        let mut rng = thread_rng();

        // let gas = BallOfGas::new(
        //     10000.0,
        //     50.0,
        //     Point3::new(0.0, 0.0, 0.0),
        //     Vector3::new(0.0, 0.0, 0.0),
        // );
        // particles.append(&mut gas.get_particles(num_particles, &mut rng));

        // let galaxy1 = Galaxy::new(
        //     1000.0,
        //     1.0,
        //     Plummer::new(1.0, 0.1),
        //     // Uniform::new(0.1, 3.0),
        //     Point3::new(0.0, 4.0, 4.0),
        //     Vector3::new(0.0, 0.0, 0.0),
        //     Vector3::new(1.0, 0.0, 0.0),
        // );
        // particles.append(&mut galaxy1.get_particles(num_particles / 2, &mut rng));

        let dim: f32 = 1.0;
        let scale: f32 = 0.1;
        let perlin = Perlin::new(1);
        while particles.len() < num_particles {
            let x: f32 = rng.gen_range(0.0..scale);
            let y: f32 = rng.gen_range(0.0..scale);
            let z: f32 = rng.gen_range(0.0..scale);

            let noise_value = perlin.get([x as f64, y as f64, z as f64]);
            let probability = (noise_value + 1.0) / 2.0; // Normalize to [0, 1]

            if rng.gen::<f64>() < probability.pow(4.0) {
                particles.push(Particle::new(
                    Point3::new(x, y, z) * (dim / scale),
                    Vector3::zero(),
                    total_mass / num_particles as f32,
                ));
            }
        }

        let dt: f32 = 0.001;
        let softening: f32 = 0.1;

        let simulation = SimulationBuffers::new(context.api().construction(), particles);

        let integrator =
            VerletIntegrator::new(simulation.clone(), dt, GRAVITATIONAL_CONSTANT, softening);
        let integrator = ComputeShaderExecutor::new(context.api().construction(), integrator);

        let energy = EnergyCalculator::new(simulation.clone(), context.api().construction());
        let energy = ComputeShaderExecutor::new(context.api().construction(), energy);

        Self {
            simulation,
            integrator,
            energy,
            render: PointCloudPipeline::new(
                context.api().construction(),
                context.viewport_subpass(),
            ),
            camera: Camera::new(),
            state: Default::default(),
        }
    }

    fn render(&mut self, info: &mut RenderInfo, api: &EngineApi) {
        if self.state.active {
            let start = Instant::now();
            self.integrator.execute(api.construction());
            self.energy.execute(api.construction());
            self.state.energy.push(self.energy.get_total_energy());
            self.state.last_simulation_time = start.elapsed();
        }

        self.render.draw(
            &self.simulation.points,
            self.camera
                .generate_view(info.viewport.dimensions[0] / info.viewport.dimensions[1]),
            self.state.brightness,
            self.state.scale,
            info,
        );
    }

    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        let width = 250.0;

        egui::SidePanel::left("left_panel")
            .min_width(width)
            .resizable(false)
            .show(context, |ui| {
                ui.separator();
                ui.heading("Newtonian Gravity Simulator");
                ui.label(format!("Using: {}", api.device_name()));
                ui.separator();

                Grid::new("render_settings")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .max_col_width(width / 2.0)
                    .min_col_width(width / 2.0)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Brightness");
                        ui.add(
                            DragValue::new(&mut self.state.brightness)
                                .speed(0.02)
                                .clamp_range(0.01..=50.0),
                        );
                        ui.end_row();
                        ui.label("Scale:");
                        ui.add(
                            DragValue::new(&mut self.state.scale)
                                .speed(0.02)
                                .clamp_range(0.0..=2.0),
                        );
                        ui.end_row();

                        ui.label("FPS");
                        ui.label(format!(
                            "{:.0}",
                            1.0 / api
                                .performance
                                .get_time("frame")
                                .unwrap_or_default()
                                .as_secs_f64()
                        ));
                        ui.end_row();
                        ui.label("UPS");
                        ui.label(format!(
                            "{:.0}",
                            1.0 / self.state.last_simulation_time.as_secs_f64()
                        ));
                        ui.end_row();

                        ui.label("Show energy:");
                        ui.checkbox(&mut self.state.show_energy, "");
                        ui.end_row()
                    });

                ui.separator();

                Grid::new("render_actions")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .max_col_width(width / 2.0)
                    .min_col_width(width / 2.0)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.add_enabled_ui(!self.state.active, |ui| {
                            ui.horizontal_centered(|ui| {
                                if ui.add(FatButton::new("Run").width(width / 2.0)).clicked() {
                                    self.state.active = true;
                                }
                            });
                        });
                        ui.add_enabled_ui(self.state.active, |ui| {
                            ui.horizontal_centered(|ui| {
                                if ui.add(FatButton::new("Stop").width(width / 2.0)).clicked() {
                                    self.state.active = false;
                                }
                            });
                        });
                    });
            });

        if self.state.show_energy {
            Window::new("Total Energy").show(context, |ctx| {
                let total: PlotPoints = self
                    .state
                    .energy
                    .iter()
                    .enumerate()
                    .map(|(i, &x)| [i as f64, x as f64])
                    .collect();
                let total = Line::new(total);
                Plot::new(format!("total_energy"))
                    .view_aspect(2.0)
                    .show(ctx, |plot_ui| {
                        plot_ui.line(total);
                        if self.state.energy.len() > 0 {
                            plot_ui.hline(
                                HLine::new(self.state.energy.last().unwrap() * 1.01)
                                    .color(Color32::WHITE),
                            );
                            plot_ui.hline(
                                HLine::new(self.state.energy.last().unwrap() * 0.99)
                                    .color(Color32::WHITE),
                            );
                        }
                    });
            });
        }
    }

    fn on_winit_event(&mut self, event: &WindowEvent, api: &mut EngineApi) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => self.on_keyboard_event(input),
            WindowEvent::MouseInput { state, button, .. } => {
                self.on_mouse_click_event(*state, *button)
            }
            WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved_event(position),
            WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel_event(delta),
            _ => {}
        }
    }
}

impl TardigradeEngine {
    fn on_keyboard_event(&mut self, input: &KeyboardInput) {
        if let Some(key_code) = input.virtual_keycode {
            match key_code {
                VirtualKeyCode::W => self.camera.forward(),
                VirtualKeyCode::S => self.camera.backward(),
                VirtualKeyCode::D => self.camera.right(),
                VirtualKeyCode::A => self.camera.left(),
                VirtualKeyCode::Q => self.camera.up(),
                VirtualKeyCode::E => self.camera.down(),
                _ => (),
            }
        }
    }

    fn on_mouse_click_event(&mut self, state: ElementState, button: MouseButton) {}

    fn on_cursor_moved_event(&mut self, position: &PhysicalPosition<f64>) {}

    fn on_mouse_wheel_event(&mut self, delta: &MouseScrollDelta) {
        let change = match delta {
            MouseScrollDelta::LineDelta(_x, y) => *y,
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
        };

        self.camera.zoom(change);
    }
}

fn main() {
    let options = EngineOptions {
        window_options: WindowOptions::default(),
        features: Features::empty(),
        ..EngineOptions::default()
    };

    EngineLauncher::<TardigradeEngine>::run(options);
}
