use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use crate::vulkan::vertex::Vertex;
use erupt::vk;
use std::mem::size_of;
use std::rc::Rc;

pub struct Model {}
pub struct Texture {}
pub struct TextureImage {}

pub struct Scene {
    models: Vec<Model>,
    textures: Vec<Texture>,
    // vertex_buffer: Buffer,
    // index_buffer: Buffer,
    // material_buffer: Buffer,
    //
    // texture_images: Vec<TextureImage>,
    // texture_image_view_handles: Vec<vk::ImageView>,
    // texture_sampler_handles: Vec<vk::Sampler>,
}

impl Scene {
    pub fn models(&self) -> &Vec<Model> {
        &self.models
    }
    pub fn textures(&self) -> &Vec<Texture> {
        &self.textures
    }
    // pub fn vertex_buffer(&self) -> &Buffer {
    //     &self.vertex_buffer
    // }
    // pub fn index_buffer(&self) -> &Buffer {
    //     &self.index_buffer
    // }
    // pub fn material_buffer(&self) -> &Buffer {
    //     &self.material_buffer
    // }
    // pub fn texture_images(&self) -> &Vec<TextureImage> {
    //     &self.texture_images
    // }
    // pub fn texture_image_view_handles(&self) -> &Vec<vk::ImageView> {
    //     &self.texture_image_view_handles
    // }
    // pub fn texture_sampler_handles(&self) -> &Vec<vk::Sampler> {
    //     &self.texture_sampler_handles
    // }

    pub fn new(device: Rc<Device>) -> Self {
        // let vertex_buffer = Buffer::new(
        //     device.clone(),
        //     3 * size_of::<Vertex>() as u64,
        //     vk::BufferUsageFlags::VERTEX_BUFFER,
        // );
        // let index_buffer = Buffer::new(
        //     device.clone(),
        //     3 * size_of::<u32>() as u64,
        //     vk::BufferUsageFlags::INDEX_BUFFER,
        // );
        // let material_buffer = Buffer::new(
        //     device.clone(),
        //     3 * size_of::<u32>() as u64,
        //     vk::BufferUsageFlags::STORAGE_BUFFER,
        // );
        Scene {
            models: vec![],
            textures: vec![],
            // vertex_buffer,
            // index_buffer,
            // material_buffer,
            // texture_images: vec![],
            // texture_image_view_handles: vec![],
            // texture_sampler_handles: vec![],
        }
    }
}
