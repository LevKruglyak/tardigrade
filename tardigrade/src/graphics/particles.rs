use std::sync::Arc;

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
        color_blend::ColorBlendState, input_assembly::InputAssemblyState, viewport::ViewportState,
    },
    GraphicsPipeline, Pipeline,
};

use crate::physics::simulation::ParticlePosition;
use super::renderer::ViewData;

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
            .input_assembly_state(InputAssemblyState::new())
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

    pub fn draw(
        &mut self,
        particles: &DeviceBuffer<ParticlePosition>,
        view: ViewData,
        info: &mut RenderInfo,
    ) {
        let mut builder = info.create_builder();

        let uniform = vs::ty::UniformData {
            world: view.world.into(),
            proj: view.proj.into(),
            view: view.view.into(),
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
