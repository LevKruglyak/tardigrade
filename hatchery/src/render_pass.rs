use std::sync::Arc;

use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    device::{Device, Queue},
    format::Format,
    image::ImageAccess,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};
use vulkano_util::{context::VulkanoContext, renderer::SwapchainImageView};

use crate::{
    engine::{Engine, EngineApi, RenderInfo},
    gui::GuiImplementation,
};

pub struct FinalRenderPass {
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
}

impl FinalRenderPass {
    pub fn new(context: &VulkanoContext, format: Format) -> Self {
        let render_pass = Self::create_render_pass(context.device().clone(), format);

        Self {
            device: context.device().clone(),
            graphics_queue: context.graphics_queue().clone(),
            render_pass,
        }
    }

    pub fn viewport_subpass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }

    pub fn ui_subpass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 1).unwrap()
    }

    fn create_render_pass(device: Arc<Device>, format: Format) -> Arc<RenderPass> {
        vulkano::ordered_passes_renderpass!(
            device,
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                }
            },
            passes: [
                { color: [color], depth_stencil: {}, input: [] }, // Draw render result
                { color: [color], depth_stencil: {}, input: [] } // Gui render pass
            ]
        )
        .expect("error creating render pass")
    }

    pub fn render<F, E>(
        &self,
        before_future: F,
        gui: &mut E::Gui,
        api: &mut EngineApi,
        subpass: Subpass,
        target: SwapchainImageView,
        engine: &mut E,
    ) -> Box<dyn GpuFuture>
    where
        F: GpuFuture + 'static,
        E: Engine + 'static,
    {
        // Get dimensions
        let image_dimensions = target.image().dimensions();

        // Create framebuffer (must be in same order as render pass description in `new`
        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![target],
                ..Default::default()
            },
        )
        .unwrap();

        // Create primary command buffer builder
        let mut primary_builder = AutoCommandBufferBuilder::primary(
            api.construction.command_allocator(),
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        // Begin render pass
        primary_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer)
                },
                SubpassContents::SecondaryCommandBuffers,
            )
            .unwrap();

        let scale_factor = api.window().scale_factor() as f32;
        let viewport = gui.viewport(scale_factor);
        let mut render_info = RenderInfo {
            command_buffer: &mut primary_builder,
            queue: self.graphics_queue.clone(),
            command_allocator: api.construction().command_allocator(),
            subpass,
            viewport,
        };

        engine.render(&mut render_info, api);

        // Render gui
        primary_builder
            .next_subpass(SubpassContents::SecondaryCommandBuffers)
            .unwrap();

        let cb = gui.render(image_dimensions.width_height());
        primary_builder.execute_commands(cb).unwrap();

        // End render pass
        primary_builder.end_render_pass().unwrap();

        // Build command buffer
        let command_buffer = primary_builder.build().unwrap();

        // Execute primary command buffer
        let after_future = before_future
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .unwrap();

        after_future.boxed()
    }
}
