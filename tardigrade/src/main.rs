#![allow(unused_variables, dead_code)]

use std::time::Instant;

use crate::graphics::view::Camera;
use cgmath::Vector3;
use egui_implementation::*;
use graphics::renderer::Renderer;
use hatchery::*;
use physics::simulation::{Particle, Simulation};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rand_distr::UnitSphere;

mod graphics;
mod physics;

pub struct GuiState {
    pub brightness: f32,
    pub size: f32,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            brightness: 0.1,
            size: 0.01,
        }
    }
}

pub struct TardigradeEngine {
    simulation: Simulation,
    renderer: Renderer,
    camera: Camera,
    state: GuiState,
    elapsed: Instant,
}

fn create_particle(rng: &mut ThreadRng) -> Particle {
    let position = Vector3::from(rng.sample(UnitSphere)) * 10.0;
    let velocity = Vector3::new(0.0, 0.0, 0.0);
    let mass = 1.0;

    Particle::new(position, velocity, mass)
}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        println!("using {}", context.api().device_name());

        let num_particles = 50_000;

        let mut rng = thread_rng();
        let particles: Vec<Particle> = (0..num_particles)
            .map(|_| create_particle(&mut rng))
            .collect();

        let simulation = Simulation::new(context.api().construction(), particles);

        Self {
            simulation,
            renderer: Renderer::new(context.api().construction(), context.viewport_subpass()),
            elapsed: Instant::now(),
            camera: Camera::new(),
            state: Default::default(),
        }
    }

    fn render(&mut self, info: &mut RenderInfo, api: &EngineApi) {
        let elapsed = self.elapsed.elapsed();

        // self.simulation.advance(api.construction());

        self.renderer.draw_particles(
            self.simulation.particles(),
            self.camera.generate_view(info.viewport.dimensions[0] / info.viewport.dimensions[1]),
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
            });
    }

    fn on_winit_event(&mut self, event: &WindowEvent, api: &mut EngineApi) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => self.on_keyboard_event(input),
            WindowEvent::MouseInput { state, button, .. } => {
                self.on_mouse_click_event(*state, *button)
            }
            // WindowEvent::CursorMoved { position, .. } => self.on_cursor_moved_event(position),
            // WindowEvent::MouseWheel { delta, .. } => self.on_mouse_wheel_event(delta),
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
                _ => (),
            }
        }
    }

     fn on_mouse_click_event(&mut self, state: ElementState, button: MouseButton) {

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
