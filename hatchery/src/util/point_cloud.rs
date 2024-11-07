use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    impl_vertex,
    pipeline::{
        graphics::{
            color_blend::ColorBlendState, input_assembly::InputAssemblyState,
            viewport::ViewportState,
        },
        GraphicsPipeline, Pipeline,
    },
    render_pass::Subpass,
};

use crate::RenderInfo;

use super::{
    buffer::{AbstractBuffer, SharedBuffer},
    camera::ViewData,
    quad::TexturedQuad,
    ConstructionContext,
};

// use crate::graphics::view::ViewData;
// use crate::physics::{ParticlePosition, ParticleVelocityMass};

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/util/point_cloud_vert.glsl",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/util/point_cloud_frag.glsl"
    }
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct RenderPoint {
    pub point_pos: [f32; 3],
}

impl_vertex!(RenderPoint, point_pos);

pub struct PointCloudPipeline {
    pipeline: Arc<GraphicsPipeline>,
    quad: TexturedQuad,
    subpass: Subpass,
}

impl PointCloudPipeline {
    pub fn new(context: &ConstructionContext, subpass: Subpass) -> Self {
        let vs = vs::load(context.device()).unwrap();
        let fs = fs::load(context.device()).unwrap();

        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(TexturedQuad::buffers_definition().instance::<RenderPoint>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .render_pass(subpass.clone())
            .color_blend_state(ColorBlendState::new(1).blend_alpha())
            .build(context.device())
            .expect("failed to make pipeline");

        let quad = TexturedQuad::new(context, [-1.0, -1.0], [1.0, 1.0]);

        Self {
            pipeline,
            quad,
            subpass,
        }
    }

    pub fn draw(
        &mut self,
        points: &SharedBuffer<RenderPoint>,
        view: ViewData,
        brightness: f32,
        size: f32,
        info: &mut RenderInfo,
    ) {
        let mut builder = info.create_builder();

        let uniform = vs::ty::UniformData {
            world: view.world.into(),
            proj: view.proj.into(),
            view: view.view.into(),
            brightness,
            size: size * view.scale,
        };

        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_vertex_buffers(0, (self.quad.vertex.buffer(), points.buffer()))
            .bind_index_buffer(self.quad.index.typed_buffer())
            .push_constants(self.pipeline.layout().clone(), 0, uniform)
            .set_viewport(0, vec![info.viewport.clone()])
            .draw_indexed(self.quad.index.len(), points.len(), 0, 0, 0)
            .unwrap();

        info.execute(builder);
    }
}
