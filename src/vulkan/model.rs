use crate::vulkan::vertex::Vertex;
use glam::{vec2, vec3};

pub struct Model {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Model {
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn triangle() -> Self {
        let vertices = vec![
            Vertex::new(
                vec3(-0.5, -0.5, 0.0),
                vec3(0.0, 0.0, -1.0),
                vec2(0.0, 0.0),
                0,
            ),
            Vertex::new(
                vec3(0.5, -0.5, 0.0),
                vec3(0.0, 0.0, -1.0),
                vec2(0.0, 0.0),
                0,
            ),
            Vertex::new(vec3(0.0, 0.5, 0.0), vec3(0.0, 0.0, -1.0), vec2(0.0, 0.0), 0),
        ];

        let indices = vec![0, 1, 2];

        Model { vertices, indices }
    }
}

pub struct DrawIndexed {
    pub vertex_offset: u64,
    pub index_offset: u64,
    pub index_count: u32,
    pub id: usize,
}
