use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::vertex::{IndexBuffer, ModelVertex, VertexBuffer};
use erupt::vk;
use glam::{Mat4, Vec3};
use gltf::buffer::Data;
use gltf::mesh::Reader;
use gltf::{Document, Semantic};
use std::mem::size_of;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Metadata {
    name: String,
    path: String,
    scene_count: usize,
    node_count: usize,
    animation_count: usize,
    skin_count: usize,
    mesh_count: usize,
    material_count: usize,
    texture_count: usize,
}

impl Metadata {
    pub fn new<P: AsRef<Path>>(path: P, document: &Document) -> Self {
        Metadata {
            name: String::from(path.as_ref().file_name().unwrap().to_str().unwrap()),
            path: String::from(path.as_ref().to_str().unwrap()),
            scene_count: document.scenes().len(),
            node_count: document.nodes().len(),
            animation_count: document.animations().len(),
            skin_count: document.skins().len(),
            mesh_count: document.meshes().len(),
            material_count: document.materials().len(),
            texture_count: document.textures().len(),
        }
    }
}

pub struct Primitive {
    index: usize,
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
}

impl Primitive {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn vertex_buffer(&self) -> &VertexBuffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Option<IndexBuffer> {
        &self.index_buffer
    }
}

pub struct Mesh {
    primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn primitives(&self) -> &Vec<Primitive> {
        &self.primitives
    }
}

impl Mesh {
    pub fn new(primitives: Vec<Primitive>) -> Self {
        Mesh { primitives }
    }
}

#[derive(Default)]
pub struct StagingBuffers {
    staging_vertex_buffer: Option<Buffer>,
    staging_index_buffer: Option<Buffer>,
}

pub struct Model {
    metadata: Metadata,
    meshes: Vec<Mesh>,
    global_transform: Mat4,
}

impl Model {
    pub fn create_from_file<P: AsRef<Path>>(
        device: Rc<Device>,
        command_buffer: CommandBuffer,
        path: P,
    ) -> (Model, StagingBuffers) {
        let (document, buffers, images) = gltf::import(&path).unwrap();

        let metadata = Metadata::new(path, &document);

        if document.scenes().len() == 0 {
            panic!("Gltf has no scenes")
        }

        let meshes = Meshes::new(device.clone(), command_buffer, &document, &buffers);

        let model = Model {
            metadata,
            meshes: meshes.meshes,
            global_transform: Default::default(),
        };

        let staging_buffers = StagingBuffers {
            staging_vertex_buffer: Some(meshes.vertices),
            staging_index_buffer: meshes.indices,
        };

        (model, staging_buffers)
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

pub struct PrimitiveData {
    index: usize,
    indices: Option<(usize, usize)>,
    vertices: (usize, usize),
}

pub struct Meshes {
    pub meshes: Vec<Mesh>,
    pub vertices: Buffer,
    pub indices: Option<Buffer>,
}

impl Meshes {
    pub fn new(
        device: Rc<Device>,
        command_buffer: CommandBuffer,
        document: &Document,
        buffers: &[Data],
    ) -> Self {
        let mut meshes_data = Vec::<Vec<PrimitiveData>>::new();
        let mut all_vertices = Vec::<ModelVertex>::new();
        let mut all_indices = Vec::<u32>::new();

        let mut primitive_count = 0;

        for mesh in document.meshes() {
            let mut primitive_buffers = Vec::<PrimitiveData>::new();

            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                if let Some(accessor) = primitive.get(&Semantic::Positions) {
                    let positions = read_positions(&reader);

                    let mut vertices = positions
                        .iter()
                        .enumerate()
                        .map(|(i, pos)| ModelVertex::new(*pos))
                        .collect::<Vec<_>>();

                    let indices = read_indices(&reader);
                    let indices = indices.map(|indices| {
                        let offset = all_indices.len() * size_of::<u32>();
                        all_indices.extend_from_slice(&indices);
                        (offset, indices.len())
                    });

                    let offset = all_vertices.len() * size_of::<ModelVertex>();
                    all_vertices.extend_from_slice(&vertices);

                    let index = primitive_count;
                    primitive_count += 1;

                    primitive_buffers.push(PrimitiveData {
                        index,
                        indices,
                        vertices: (offset, accessor.count()),
                    });
                }
            }
            meshes_data.push(primitive_buffers);
        }

        if meshes_data.is_empty() {
            panic!()
        }

        let buffer_usage_flags = vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS
            | vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
        let mut indices = if all_indices.is_empty() {
            None
        } else {
            let (index_buffer, staging_index_buffer) = command_buffer
                .create_device_local_buffer_with_data::<_>(
                    device.clone(),
                    vk::BufferUsageFlags::INDEX_BUFFER | buffer_usage_flags,
                    &all_indices,
                );
            Some((Arc::new(index_buffer), staging_index_buffer))
        };

        let (vertex_buffer, staging_vertex_buffer) = command_buffer
            .create_device_local_buffer_with_data::<_>(
                device.clone(),
                vk::BufferUsageFlags::VERTEX_BUFFER | buffer_usage_flags,
                &all_vertices,
            );
        let vertex_buffer = Arc::new(vertex_buffer);

        let meshes = meshes_data
            .iter()
            .map(|primitives_data| {
                let primitives = primitives_data
                    .iter()
                    .map(|primitive| {
                        let mesh_vertices = primitive.vertices;
                        let vertex_buffer = VertexBuffer::new(
                            vertex_buffer.clone(),
                            mesh_vertices.0 as _,
                            mesh_vertices.1 as _,
                        );

                        let index_buffer = primitive.indices.map(|mesh_indices| {
                            IndexBuffer::new(
                                indices.as_ref().unwrap().0.clone(),
                                mesh_indices.0 as _,
                                mesh_indices.1 as _,
                            )
                        });

                        Primitive {
                            index: primitive.index,
                            vertex_buffer,
                            index_buffer,
                        }
                    })
                    .collect::<Vec<_>>();
                Mesh::new(primitives)
            })
            .collect::<Vec<_>>();

        Meshes {
            meshes,
            vertices: staging_vertex_buffer,
            indices: indices.map(|(_, staging_index_buffer)| staging_index_buffer),
        }
    }
}

fn read_positions<'a, 's, F>(reader: &Reader<'a, 's, F>) -> Vec<Vec3>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_positions()
        .unwrap()
        .map(|pos| Vec3::from(pos))
        .collect()
}

fn read_indices<'a, 's, F>(reader: &Reader<'a, 's, F>) -> Option<Vec<u32>>
where
    F: Clone + Fn(gltf::Buffer<'a>) -> Option<&'s [u8]>,
{
    reader
        .read_indices()
        .map(|indices| indices.into_u32().collect::<Vec<_>>())
}
