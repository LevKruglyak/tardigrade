#![allow(unused_variables, unused_imports, dead_code)]

use std::{f32::consts::TAU, sync::Arc};

use cgmath::{InnerSpace, Point3, Vector3};
use egui::{
    plot::{HLine, Line, Plot, PlotPoints},
    Color32, DragValue, Grid, Window,
};
use egui_implementation::*;
use hatchery::{
    dpi::PhysicalPosition,
    event::{
        ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
    },
    util::compute::ComputeShaderExecutor,
    *,
};
use rand::{thread_rng, Rng};
use rand_distr::{Uniform, UnitBall, UnitCircle};
use util::{buffer::AbstractBuffer, camera::Camera, point_cloud::PointCloudPipeline};

pub struct GuiState {}

impl Default for GuiState {
    fn default() -> Self {
        Self {}
    }
}

pub struct TardigradeEngine {
    state: GuiState,
}

impl Engine for TardigradeEngine {
    type Gui = EguiImplementation;

    fn init(context: &mut EngineContext<Self::Gui>) -> Self {
        Self {
            state: Default::default(),
        }
    }

    fn render(&mut self, info: &mut RenderInfo, api: &EngineApi) {}

    fn immediate(&mut self, context: &mut egui::Context, api: &mut EngineApi) {
        let width = 250.0;

        egui::SidePanel::left("left_panel")
            .min_width(width)
            .resizable(false)
            .show(context, |ui| {
                ui.separator();
                ui.heading("Hatchery Example");
                ui.label(format!("Using: {}", api.device_name()));
                ui.separator();
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
                // VirtualKeyCode::W => self.camera.forward(),
                // VirtualKeyCode::S => self.camera.backward(),
                // VirtualKeyCode::D => self.camera.right(),
                // VirtualKeyCode::A => self.camera.left(),
                // VirtualKeyCode::Q => self.camera.up(),
                // VirtualKeyCode::E => self.camera.down(),
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

        // self.camera.zoom(change);
    }
}

fn main() {
    let options = EngineOptions::default();
    EngineLauncher::<TardigradeEngine>::run(options);
}
