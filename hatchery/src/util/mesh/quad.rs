use super::{GenericMesh, Vertex2};

pub struct Quad;

impl GenericMesh<Vertex2> for Quad {
    fn vertices() -> Vec<Vertex2> {
        vec![
            Vertex2 {
                vertex_pos: [min[0], min[1]],
                vertex_uv: [0.0, 0.0],
            },
            Vertex2 {
                vertex_pos: [min[0], max[1]],
                vertex_uv: [0.0, 1.0],
            },
            Vertex2 {
                vertex_pos: [max[0], max[1]],
                vertex_uv: [1.0, 1.0],
            },
            Vertex2 {
                vertex_pos: [max[0], min[1]],
                vertex_uv: [1.0, 0.0],
            },
        ]
    }

    fn indices() -> Vec<u32> {}
}
