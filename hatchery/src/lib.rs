#![allow(unused_variables, dead_code)]

mod gui;
pub mod util;

mod engine;
mod performance;
mod render_pass;

pub use engine::Engine;
pub use engine::EngineApi;
pub use engine::EngineContext;
pub use engine::EngineLauncher;
pub use engine::EngineOptions;
pub use engine::WindowOptions;
pub use engine::RenderInfo;
pub use gui::GuiImplementation;

// pub extern crate vulkano;
// pub extern crate vulkano_util;
// pub extern crate vulkano_shaders;
// pub extern crate bytemuck;

pub use vulkano::command_buffer::PrimaryAutoCommandBuffer;
pub use vulkano::render_pass::Subpass;
pub use vulkano::{
    command_buffer::AutoCommandBufferBuilder, pipeline::graphics::viewport::Viewport,
};
pub use vulkano::device::Features;
pub use winit::dpi::LogicalSize;

#[cfg(feature = "egui")]
pub mod egui_implementation {
    pub use crate::gui::egui::EguiImplementation;
    pub use egui;
}
