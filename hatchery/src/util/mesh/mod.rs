use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::BufferUsage,
    impl_vertex,
    pipeline::graphics::vertex_input::{BuffersDefinition, Vertex},
};

pub mod cube;
pub mod quad;

use super::{
    buffer::{AbstractBuffer, SharedBuffer},
    ConstructionContext,
};

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct Vertex2 {
    vertex_pos: [f32; 2],
    vertex_uv: [f32; 2],
}

#[repr(C)]
#[derive(Default, Pod, Zeroable, Clone, Copy)]
pub struct Vertex3 {
    vertex_pos: [f32; 3],
    vertex_uv: [f32; 3],
}

impl_vertex!(Vertex2, vertex_pos, vertex_uv);
impl_vertex!(Vertex3, vertex_pos, vertex_uv);

pub struct Mesh<V: Vertex> {
    pub vertex: SharedBuffer<V>,
    pub index: SharedBuffer<u32>,
}

pub trait GenericMesh<V: Vertex> {
    fn vertices() -> Vec<V>;
    fn indices() -> Vec<u32>;
}

impl<V: Vertex> Mesh<V> {
    pub fn new(context: &ConstructionContext, vertex: Vec<V>, index: Vec<u32>) -> Self {
        let vertex = SharedBuffer::from_iter(
            context,
            BufferUsage {
                vertex_buffer: true,
                ..BufferUsage::empty()
            },
            vertex,
        );

        let index = SharedBuffer::from_iter(
            context,
            BufferUsage {
                index_buffer: true,
                ..BufferUsage::empty()
            },
            index,
        );

        Self { vertex, index }
    }

    pub fn buffers_definition() -> BuffersDefinition {
        BuffersDefinition::new().vertex::<V>()
    }
}
