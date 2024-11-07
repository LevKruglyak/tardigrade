#![allow(unused_variables, dead_code)]

use cgmath::Vector3;
use egui_implementation::*;
use hatchery::{
    dpi::PhysicalPosition,
    event::{
        ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    },
    util::compute::ComputeShaderExecutor,
    *,
};
use physics::{verlet::Simulation, Particle};
use util::{buffer::AbstractBuffer, camera::Camera, point_cloud::PointCloudPipeline};

mod physics;

pub struct GuiState {}

impl Default for GuiState {
    fn default() -> Self {
        Self {}
    }
}

pub struct TardigradeEngine {
    simulation: ComputeShaderExecutor<Simulation>,
    render: PointCloudPipeline,
    camera: Camera,
    state: GuiState,
}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        let mut particles: Vec<Particle> = Vec::new();
        particles.push(Particle::new(
            Vector3::new(10.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            1.0,
        ));
        particles.push(Particle::new(
            Vector3::new(0.0, 10.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            1.0,
        ));
        particles.push(Particle::new(
            Vector3::new(0.0, 0.0, 10.0),
            Vector3::new(0.0, 0.0, 0.0),
            1.0,
        ));
        particles.push(Particle::new(
            Vector3::new(10.0, 10.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            1.0,
        ));
        particles.push(Particle::new(
            Vector3::new(10.0, 0.0, 10.0),
            Vector3::new(0.0, 0.0, 0.0),
            1.0,
        ));

        let simulation = Simulation::new(context.api().construction(), particles);
        let executor = ComputeShaderExecutor::new(context.api().construction(), simulation);

        Self {
            simulation: executor,
            render: PointCloudPipeline::new(
                context.api().construction(),
                context.viewport_subpass(),
            ),
            camera: Camera::new(),
            state: Default::default(),
        }
    }

    fn render(&mut self, info: &mut RenderInfo, api: &EngineApi) {
        // if self.state.active {
        //     let start = Instant::now();
        //     for _ in 0..10 {
        //         self.simulation.execute(api.construction());
        //     }
        //     self.simulation.shader_mut().calculate_energy();
        //     self.last_time = start.elapsed();
        // }

        self.render.draw(
            self.simulation.shader().particles(),
            self.camera
                .generate_view(info.viewport.dimensions[0] / info.viewport.dimensions[1]),
            1.0,
            2.0,
            info,
        )
        // self.renderer.draw_particles(
        //     self.simulation.shader().particles(),
        //     self.simulation.shader().velocity_mass(),
        //     self.camera
        //         .generate_view(info.viewport.dimensions[0] / info.viewport.dimensions[1]),
        //     self.state.brightness,
        //     10.0 * self.state.size,
        //     info,
        // );
    }

    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        egui::SidePanel::left("left_panel")
            .min_width(250.0)
            .resizable(false)
            .show(context, |ui| {
                ui.heading("Newtonian Gravity Simulator");
                ui.label(format!("Using: {}", api.device_name()));
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
