#![allow(unused_variables, dead_code)]

use std::time::{Duration, Instant};

use crate::graphics::view::Camera;
use cgmath::{Vector3, num_traits::Float};
use egui_implementation::*;
use graphics::renderer::Renderer;
use hatchery::{
    dpi::PhysicalPosition,
    event::{
        ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    },
    *,
};
use physics::simulation::{Particle, Simulation};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::{UnitBall, UnitSphere, uniform::SampleUniform, Distribution, Uniform};

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
    simulation: Simulation,
    renderer: Renderer,
    camera: Camera,
    state: GuiState,
    last_time: Duration,
}

pub struct UnitShell;

impl<F: Float + SampleUniform> Distribution<[F; 3]> for UnitShell {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [F; 3] {
        let uniform = Uniform::new(F::from(-1.).unwrap(), F::from(1.).unwrap());
        let mut x1;
        let mut x2;
        let mut x3;
        loop {
            x1 = uniform.sample(rng);
            x2 = uniform.sample(rng);
            x3 = uniform.sample(rng);
            let ra = x1 * x1 + x2 * x2 + x3 * x3;
            if ra <= F::from(1.).unwrap() && ra >= F::from(0.9).unwrap() {
                break;
            }
        }
        [x1, x2, x3]
    }
}
fn create_particle(rng: &mut ThreadRng) -> Particle {
    let position = Vector3::from(rng.sample(UnitShell)) * 10.0;
    let velocity = Vector3::new(0.0, 0.0, 0.0);
    let mass = 0.0000000005;

    Particle::new(position, velocity, mass)
}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        println!("using {}", context.api().device_name());

        let num_particles = 400_000;

        let mut rng = thread_rng();

        let particles: Vec<Particle> = (0..num_particles)
            .map(|_| create_particle(&mut rng))
            .collect();

        // particles.insert(0, Particle::new(Vector3::new(4.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 0.0), 0.0002));

        let simulation = Simulation::new(context.api().construction(), particles);

        Self {
            simulation,
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
                self.simulation.advance(api.construction());
            }
            self.last_time = start.elapsed();
        }

        self.renderer.draw_particles(
            self.simulation.particles(),
            self.simulation.velocity_mass(),
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

                ui.label(format!("last time: {} ms", self.last_time.as_millis()));
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
