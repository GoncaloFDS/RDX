use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::vertex::{IndexBuffer, ModelVertex, VertexBuffer};
use erupt::vk;
use glam::{vec3, Mat4, Vec3};
use gltf::buffer::Data;
use gltf::mesh::Reader;
use gltf::{Document, Gltf, Semantic};
use std::mem::size_of;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

pub struct Mesh {
    vertices: Vec<ModelVertex>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn vertices(&self) -> &[ModelVertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}

pub struct Model {
    meshes: Vec<Mesh>,
    global_transform: Mat4,
}

impl Model {
    pub fn cube() -> Model {
        let p0 = vec3(-0.5, -0.5, -0.5);
        let p1 = vec3(0.5, 0.5, 0.5);
        let vertices = vec![
            //
            ModelVertex::new(vec3(p0.x, p0.y, p0.z)),
            ModelVertex::new(vec3(p0.x, p0.y, p1.z)),
            ModelVertex::new(vec3(p0.x, p1.y, p1.z)),
            ModelVertex::new(vec3(p0.x, p1.y, p0.z)),
            //
            ModelVertex::new(vec3(p1.x, p0.y, p1.z)),
            ModelVertex::new(vec3(p1.x, p0.y, p0.z)),
            ModelVertex::new(vec3(p1.x, p1.y, p0.z)),
            ModelVertex::new(vec3(p1.x, p1.y, p1.z)),
            //
            ModelVertex::new(vec3(p1.x, p0.y, p0.z)),
            ModelVertex::new(vec3(p0.x, p0.y, p0.z)),
            ModelVertex::new(vec3(p0.x, p1.y, p0.z)),
            ModelVertex::new(vec3(p1.x, p1.y, p0.z)),
            //
            ModelVertex::new(vec3(p0.x, p0.y, p1.z)),
            ModelVertex::new(vec3(p1.x, p0.y, p1.z)),
            ModelVertex::new(vec3(p1.x, p1.y, p1.z)),
            ModelVertex::new(vec3(p0.x, p1.y, p1.z)),
            //
            ModelVertex::new(vec3(p0.x, p0.y, p0.z)),
            ModelVertex::new(vec3(p1.x, p0.y, p0.z)),
            ModelVertex::new(vec3(p1.x, p0.y, p1.z)),
            ModelVertex::new(vec3(p0.x, p0.y, p1.z)),
            //
            ModelVertex::new(vec3(p1.x, p1.y, p0.z)),
            ModelVertex::new(vec3(p0.x, p1.y, p0.z)),
            ModelVertex::new(vec3(p0.x, p1.y, p1.z)),
            ModelVertex::new(vec3(p1.x, p1.y, p1.z)),
        ];

        let indices = vec![
            0, 1, 2, //
            0, 2, 3, //
            4, 5, 6, //
            4, 6, 7, //
            8, 9, 10, //
            8, 10, 11, //
            12, 13, 14, //
            12, 14, 15, //
            16, 17, 18, //
            16, 18, 19, //
            20, 21, 22, //
            20, 22, 23, //
        ];

        let mesh = Mesh { vertices, indices };

        Model {
            meshes: vec![mesh],
            global_transform: Default::default(),
        }
    }
}

impl Model {
    pub fn mesh(&self, index: usize) -> &Mesh {
        &self.meshes[index]
    }

    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }
}
