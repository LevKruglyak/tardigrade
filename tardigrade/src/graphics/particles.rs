use std::sync::Arc;

use cgmath::{Matrix4, Vector3, Point3, Rad, Matrix3};
use hatchery::{
    util::{
        buffer::{AbstractBuffer, DeviceBuffer},
        quad::TexturedQuad,
        ConstructionContext,
    },
    RenderInfo, Subpass,
};
use vulkano::pipeline::{
    graphics::{
        color_blend::ColorBlendState,
        input_assembly::InputAssemblyState,
        viewport::ViewportState,
    },
    GraphicsPipeline, Pipeline,
};

use crate::physics::simulation::ParticlePosition;
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/graphics/part_v.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/graphics/part_f.glsl"
    }
}
pub struct ParticlesPipeline {
    pipeline: Arc<GraphicsPipeline>,
    particle: TexturedQuad,
    subpass: Subpass,
}

impl ParticlesPipeline {
    pub fn new(context: &ConstructionContext, subpass: Subpass) -> Self {
        let vs = vs::load(context.device()).unwrap();
        let fs = fs::load(context.device()).unwrap();

        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(TexturedQuad::buffers_definition().instance::<ParticlePosition>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state( InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .render_pass(subpass.clone())
            .color_blend_state(ColorBlendState::new(1).blend_alpha())
            .build(context.device())
            .expect("failed to make pipeline");

        let particle = TexturedQuad::new(context, [0.0, 0.0], [1.0, 1.0]);

        Self {
            pipeline,
            particle,
            subpass,
        }
    }

    pub fn draw(&mut self, particles: &DeviceBuffer<ParticlePosition>, info: &mut RenderInfo) {
        let mut builder = info.create_builder();

        let uniform = {
            let rotation = 0.0;
                // 10.0 * (elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1_000_000_000.0);
            let rotation = Matrix3::from_angle_y(Rad(0.1 * rotation as f32));

            let aspect_ratio = info.viewport.dimensions[0] / info.viewport.dimensions[1];
            let proj =
                cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), aspect_ratio, 0.01, 100.0);
            let view = Matrix4::look_at_rh(
                Point3::new(0.3, 0.3, 1.0),
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
            );
            let scale = Matrix4::from_scale(0.01);

            vs::ty::UniformData {
                world: Matrix4::from(rotation).into(),
                view: (view * scale).into(),
                proj: proj.into(),
            }
        };

        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_vertex_buffers(0, (self.particle.vertex.buffer(), particles.buffer()))
            .bind_index_buffer(self.particle.index.typed_buffer())
            .push_constants(self.pipeline.layout().clone(), 0, uniform)
            .set_viewport(0, vec![info.viewport.clone()])
            .draw_indexed(self.particle.index.len(), particles.len(), 0, 0, 0)
            .unwrap();

        info.execute(builder);
    }
}
