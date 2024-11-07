use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::BufferUsage, impl_vertex, pipeline::graphics::vertex_input::BuffersDefinition,
};

use super::{
    buffer::{AbstractBuffer, SharedBuffer},
    ConstructionContext,
};

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct QuadVertex {
    quad_pos: [f32; 2],
    quad_uv: [f32; 2],
}

impl_vertex!(QuadVertex, quad_pos, quad_uv);

pub struct TexturedQuad {
    pub vertex: SharedBuffer<QuadVertex>,
    pub index: SharedBuffer<u32>,
}

impl TexturedQuad {
    pub fn new(context: &ConstructionContext, min: [f32; 2], max: [f32; 2]) -> Self {
        let vertex = SharedBuffer::from_vec(
            context,
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            vec![
                QuadVertex {
                    quad_pos: [min[0], min[1]],
                    quad_uv: [0.0, 0.0],
                },
                QuadVertex {
                    quad_pos: [min[0], max[1]],
                    quad_uv: [0.0, 1.0],
                },
                QuadVertex {
                    quad_pos: [max[0], max[1]],
                    quad_uv: [1.0, 1.0],
                },
                QuadVertex {
                    quad_pos: [max[0], min[1]],
                    quad_uv: [1.0, 0.0],
                },
            ],
        );

        let index = SharedBuffer::from_vec(
            context,
            BufferUsage {
                index_buffer: true,
                ..BufferUsage::empty()
            },
            vec![0, 2, 1, 0, 3, 2],
        );

        Self { vertex, index }
    }

    pub fn buffers_definition() -> BuffersDefinition {
        BuffersDefinition::new().vertex::<QuadVertex>()
    }
    //
    // pub fn draw(&self, builder: &mut AutoCommandBufferBuilder<SecondaryAutoCommandBuffer>) {
    //     builder
    //         .bind_vertex_buffers(0, self.vertex.buffer())
    //         .bind_index_buffer(self.index.buffer())
    //         .draw_indexed(self.index.len() as u32, 1, 0, 0, 0)
    //         .expect("failed to draw quad");
    // }
}
