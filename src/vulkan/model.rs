use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::vertex::{IndexBuffer, ModelVertex, Std430ModelVertex, VertexBuffer};
use crevice::std430::AsStd430;
use erupt::vk;
use glam::{vec2, vec3, Mat4, Vec3};
use gltf::buffer::Data;
use gltf::mesh::Reader;
use gltf::{Document, Gltf, Semantic};
use std::mem::size_of;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

pub struct Instance {
    id: u32,
    blas_id: u32,
    transform: Mat4,
}

impl Instance {
    pub fn new(id: u32, blas_id: u32, transform: Mat4) -> Self {
        Instance {
            id,
            blas_id,
            transform,
        }
    }
}

impl Instance {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn blas_id(&self) -> u32 {
        self.blas_id
    }

    pub fn transform(&self) -> Mat4 {
        self.transform
    }
}

pub struct Mesh {
    vertices: Vec<Std430ModelVertex>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn vertices(&self) -> &[Std430ModelVertex] {
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
            ModelVertex::new(vec3(p0.x, p0.y, p0.z), vec2(0.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p0.y, p1.z), vec2(1.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p1.y, p1.z), vec2(1.0, 0.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p1.y, p0.z), vec2(0.0, 0.0)).as_std430(),
            //
            ModelVertex::new(vec3(p1.x, p0.y, p1.z), vec2(0.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p0.y, p0.z), vec2(1.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p1.y, p0.z), vec2(1.0, 0.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p1.y, p1.z), vec2(0.0, 0.0)).as_std430(),
            //
            ModelVertex::new(vec3(p1.x, p0.y, p0.z), vec2(0.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p0.y, p0.z), vec2(1.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p1.y, p0.z), vec2(1.0, 0.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p1.y, p0.z), vec2(0.0, 0.0)).as_std430(),
            //
            ModelVertex::new(vec3(p0.x, p0.y, p1.z), vec2(0.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p0.y, p1.z), vec2(1.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p1.y, p1.z), vec2(1.0, 0.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p1.y, p1.z), vec2(0.0, 0.0)).as_std430(),
            //
            ModelVertex::new(vec3(p0.x, p0.y, p0.z), vec2(0.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p0.y, p0.z), vec2(1.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p0.y, p1.z), vec2(1.0, 0.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p0.y, p1.z), vec2(0.0, 0.0)).as_std430(),
            //
            ModelVertex::new(vec3(p1.x, p1.y, p0.z), vec2(0.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p1.y, p0.z), vec2(1.0, 1.0)).as_std430(),
            ModelVertex::new(vec3(p0.x, p1.y, p1.z), vec2(1.0, 0.0)).as_std430(),
            ModelVertex::new(vec3(p1.x, p1.y, p1.z), vec2(0.0, 0.0)).as_std430(),
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
