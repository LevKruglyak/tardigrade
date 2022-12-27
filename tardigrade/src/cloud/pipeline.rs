use std::sync::Arc;
use std::time::Instant;

use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix3, Rad, Matrix4, Point3, Vector3};
use tardigrade_launcher::{Subpass, Viewport, AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::command_buffer::{CommandBufferUsage, CommandBufferInheritanceInfo};
use vulkano::impl_vertex;
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::input_assembly::{InputAssemblyState, PrimitiveTopology};
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::{
    buffer::DeviceLocalBuffer,
    device::{Device, Queue},
    pipeline::{graphics::vertex_input::BuffersDefinition, GraphicsPipeline},
};

#[allow(clippy::needless_question_mark)]
mod vs {
    use vulkano_shaders::shader;

    shader! {
        ty: "vertex",
        path: "src/cloud/vert.glsl"
    }
}

#[allow(clippy::needless_question_mark)]
mod fs {
    use vulkano_shaders::shader;

    shader! {
        ty: "fragment",
        path: "src/cloud/frag.glsl"
    }
}

pub struct CloudPipeline {
    device: Arc<Device>,
    graphics_pipeline: Arc<GraphicsPipeline>,
    subpass: Subpass,
    graphics_queue: Arc<Queue>,
    cloud_buffer: Arc<DeviceLocalBuffer<[[f32; 4]]>>,
    num_particles: usize,
    start: Instant,
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
struct CloudVertex {
    position: [f32; 4],
}

impl_vertex!(CloudVertex, position);

impl CloudPipeline {
    pub fn new(
        graphics_queue: Arc<Queue>,
        subpass: Subpass,
        cloud_buffer: Arc<DeviceLocalBuffer<[[f32; 4]]>>,
        num_particles: usize,
    ) -> Self {
        let graphics_pipeline = {
            let vs =
                vs::load(graphics_queue.device().clone()).expect("failed to create shader module");
            let fs =
                fs::load(graphics_queue.device().clone()).expect("failed to create shader module");

            GraphicsPipeline::start()
                .vertex_input_state(BuffersDefinition::new().vertex::<CloudVertex>())
                .vertex_shader(vs.entry_point("main").unwrap(), ())
                .fragment_shader(fs.entry_point("main").unwrap(), ())
                .input_assembly_state(
                    InputAssemblyState::new().topology(PrimitiveTopology::PointList),
                )
                .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
                // .line_width(2.0)
                .render_pass(subpass.clone())
                .color_blend_state(ColorBlendState::new(1).blend_alpha())
                .build(graphics_queue.device().clone())
                .expect("failed to make pipeline")
        };

        Self {
            device: graphics_queue.device().clone(),
            graphics_queue,
            cloud_buffer,
            subpass,
            graphics_pipeline,
            num_particles,
            start: Instant::now(),
        }
    }

    pub fn draw(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        viewport: Viewport,
    ) {
        let mut secondary_builder = AutoCommandBufferBuilder::secondary(
            self.device.clone(),
            self.graphics_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
            CommandBufferInheritanceInfo {
                render_pass: Some(self.subpass.clone().into()),
                ..Default::default()
            },
        )
        .unwrap();

        let uniform_data = {
            let elapsed = self.start.elapsed();
            let rotation = 0.0;
                // elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0;
            let rotation = Matrix3::from_angle_y(Rad(0.1 * rotation as f32));

            // note: this teapot was meant for OpenGL where the origin is at the lower left
            //       instead the origin is at the upper left in Vulkan, so we reverse the Y axis
            let aspect_ratio = viewport.dimensions[0] as f32 / viewport.dimensions[1] as f32;
            let proj = cgmath::perspective(
                Rad(std::f32::consts::FRAC_PI_2),
                aspect_ratio,
                0.01,
                100.0,
            );
            let view = Matrix4::look_at(
                Point3::new(0.3, 0.3, 1.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            );
            let scale = Matrix4::from_scale(0.01);

            vs::ty::Data {
                world: Matrix4::from(rotation).into(),
                view: (view * scale).into(),
                proj: proj.into(),
            }
        };

        secondary_builder
            .bind_pipeline_graphics(self.graphics_pipeline.clone())
            .bind_vertex_buffers(0, self.cloud_buffer.clone())
            .push_constants(
                self.graphics_pipeline.layout().clone(),
                0,
                uniform_data,
            )
            .set_viewport(0, vec![viewport])
            .draw(self.num_particles as u32, 1, 0, 0)
            .unwrap();
        command_buffer
            .execute_commands(secondary_builder.build().unwrap())
            .unwrap();
    }
}