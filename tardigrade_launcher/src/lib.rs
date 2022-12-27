#![allow(unused_variables, dead_code)]

mod gui;

mod engine;
mod performance;
mod render_pass;

pub use engine::Engine;
pub use engine::EngineApi;
pub use engine::EngineContext;
pub use engine::EngineLauncher;
pub use engine::EngineOptions;
pub use engine::WindowOptions;
pub use gui::GuiImplementation;

pub use vulkano;
pub use vulkano_util;
pub use vulkano_shaders;

pub use vulkano::command_buffer::PrimaryAutoCommandBuffer;
pub use vulkano::render_pass::Subpass;
pub use vulkano::{
    command_buffer::AutoCommandBufferBuilder, pipeline::graphics::viewport::Viewport,
};
pub use winit::dpi::LogicalSize;

#[cfg(feature = "egui")]
pub mod gui_implementation {
    pub use crate::gui::egui_implementation::EguiImplementation;
    pub use egui;
}
