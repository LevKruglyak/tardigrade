#![allow(unused_variables, dead_code)]

use std::{
    f32::consts::{self, PI, TAU},
    time::{Duration, Instant},
};

use crate::graphics::view::Camera;
use cgmath::{num_traits::Float, Vector3, Zero};
use egui_implementation::*;
use graphics::renderer::Renderer;
use hatchery::{
    dpi::PhysicalPosition,
    egui_implementation::egui::plot::{Line, Plot, PlotPoints},
    event::{
        ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    },
    util::compute::ComputeShaderExecutor,
    *,
};
use physics::{bh_simulation::BhSimulationShader, standard_simulation::SimulationShader, Particle};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::{uniform::SampleUniform, Distribution, Uniform, UnitBall, UnitSphere};

mod graphics;
mod physics;

pub struct GuiState {
    pub brightness: f32,
    pub size: f32,
    pub active: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            brightness: 0.1,
            size: 0.01,
            active: false,
        }
    }
}

pub struct TardigradeEngine {
    simulation: ComputeShaderExecutor<SimulationShader>,
    renderer: Renderer,
    camera: Camera,
    state: GuiState,
    last_time: Duration,
}

pub struct DiskGalaxy;

impl DiskGalaxy {
    fn black_hole() -> Particle {
        Particle::new(Vector3::zero(), Vector3::zero(), 0.1)
    }
}

impl Distribution<Particle> for DiskGalaxy {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Particle {
        let a = 1.0;

        let theta = rng.gen_range(0.0..TAU);
        let r = rng.gen_range(1.5..12.0);

        let mass = 0.0001;
        let position = Vector3::new(0.0, r * theta.cos(), r * theta.sin());

        let rotation = -1.0;
        // let v = (0.1 / r).sqrt();
        let v = 0.0 * (0.1 / r).sqrt();
        let velocity = Vector3::new(0.0, rotation * v * theta.sin(), -rotation * v * theta.cos());

        Particle::new(position, velocity, mass)
    }
}

pub struct PlummerDistribution;

impl Distribution<Particle> for PlummerDistribution {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Particle {
        let a = 1.0;

        let phi = rng.gen_range(0.0..TAU);
        let theta = rng.gen_range(-1.0..1.0).acos();
        let r = a / (rng.gen_range(0.0..1.0).powf(-0.666666) - 1.0).sqrt();

        let position = Vector3::new(0.0, r * phi.cos(), r * phi.sin());

        let mut s = 0.0;
        let mut t = 0.1;
        while t > s * s * (1.0 - s * s).powf(3.5) {
            s = rng.gen_range(0.0..1.0);
            t = rng.gen_range(0.0..0.1);
        }

        let v = 100.0 * s * 2.0.sqrt() * (1.0 + r * r).powf(-0.25);
        let phi = rng.gen_range(0.0..TAU);
        let theta = rng.gen_range(-1.0..1.0).acos();

        let velocity = 0.0001 * Vector3::new(0.0, v * phi.cos(), v * phi.sin());
        let mass = 0.0001;

        Particle::new(position, velocity, mass)
    }
}

pub struct BallOfGas;

impl Distribution<Particle> for BallOfGas {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Particle {
        let position = Vector3::from(rng.sample(UnitBall)) * 10.0;
        let velocity = Vector3::new(0.0, 0.0, 0.0);
        let mass = 0.0001;

        Particle::new(position, velocity, mass)
    }
}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        println!("using {}", context.api().device_name());

        let num_particles = 100_000;

        let mut rng = thread_rng();

        let mut particles: Vec<Particle> =
            (0..num_particles).map(|_| rng.sample(BallOfGas)).collect();
        // particles.insert(0, DiskGalaxy::black_hole());

        let simulation = SimulationShader::new(context.api().construction(), particles);
        let executor = ComputeShaderExecutor::new(context.api().construction(), simulation);

        Self {
            simulation: executor,
            renderer: Renderer::new(context.api().construction(), context.viewport_subpass()),
            last_time: Default::default(),
            camera: Camera::new(),
            state: Default::default(),
        }
    }

    fn render(&mut self, info: &mut RenderInfo, api: &EngineApi) {
        if self.state.active {
            let start = Instant::now();
            for _ in 0..10 {
                self.simulation.execute(api.construction());
            }
            self.simulation.shader_mut().calculate_energy();
            self.last_time = start.elapsed();
        }

        self.renderer.draw_particles(
            self.simulation.shader().particles(),
            self.simulation.shader().velocity_mass(),
            self.camera
                .generate_view(info.viewport.dimensions[0] / info.viewport.dimensions[1]),
            self.state.brightness,
            10.0 * self.state.size,
            info,
        );
    }

    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        egui::SidePanel::left("left_panel")
            .min_width(400.0)
            .resizable(false)
            .show(context, |ui| {
                ui.label("Size:");
                ui.add(egui::Slider::new(&mut self.state.size, 0.001..=0.1))
                    .changed();

                ui.label("Brightness:");
                ui.add(egui::Slider::new(&mut self.state.brightness, 0.01..=1.0))
                    .changed();

                if ui.button("Run").clicked() {
                    self.state.active = !self.state.active;
                }

                ui.label(format!("last time: {} us", self.last_time.as_micros()));

                let kinetic: PlotPoints = self
                    .simulation
                    .shader()
                    .kinetic_energy
                    .iter()
                    .enumerate()
                    .map(|(i, x)| [i as f64, *x as f64])
                    .collect();
                let potential: PlotPoints = self
                    .simulation
                    .shader()
                    .potential_energy
                    .iter()
                    .enumerate()
                    .map(|(i, x)| [i as f64, *x as f64])
                    .collect();
                let total: PlotPoints = self
                    .simulation
                    .shader()
                    .potential_energy
                    .iter()
                    .zip(self.simulation.shader().kinetic_energy.iter())
                    .map(|(x, y)| x + y)
                    .enumerate()
                    .map(|(i, x)| [i as f64, x as f64])
                    .collect();
                let line = Line::new(kinetic);
                let line2 = Line::new(potential);
                let line3 = Line::new(total);
                Plot::new(format!("{}", thread_rng().gen_range(0.0..1.0)))
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                    });
                Plot::new(format!("{}", thread_rng().gen_range(0.0..1.0)))
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| {
                        plot_ui.line(line2);
                    });
                Plot::new(format!("{}", thread_rng().gen_range(0.0..1.0)))
                    .view_aspect(2.0)
                    .show(ui, |plot_ui| {
                        plot_ui.line(line3);
                    });
            });
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
