use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::model::Model;
use crate::vulkan::texture::Texture;
use std::rc::Rc;

pub struct Scene {
    models: Vec<Model>,
    textures: Vec<Texture>,
}

impl Scene {
    pub fn models(&self) -> &Vec<Model> {
        &self.models
    }
    pub fn textures(&self) -> &Vec<Texture> {
        &self.textures
    }

    pub fn new() -> Self {
        let models = vec![Model::cube()];
        Scene {
            models,
            textures: vec![],
        }
    }

    pub fn insert(&mut self, model: Model) {
        self.models.push(model)
    }
}
