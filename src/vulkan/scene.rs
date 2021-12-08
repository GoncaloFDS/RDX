use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::model::{Instance, Model};
use crate::vulkan::texture::Texture;
use glam::{vec3, Mat4};
use std::rc::Rc;

pub struct Scene {
    instances: Vec<Instance>,
    models: Vec<Model>,
    textures: Vec<Texture>,
}

impl Scene {
    pub fn instances(&self) -> &[Instance] {
        &self.instances
    }

    pub fn models(&self) -> &[Model] {
        &self.models
    }

    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }
}

impl Scene {
    pub fn new() -> Self {
        let models = vec![Model::cube()];
        let mut instances = vec![];

        for i in 0..100 {
            for j in 0..100 {
                instances.push(Instance::new(
                    0,
                    Mat4::from_translation(vec3(2.0 * i as f32 - 50.0, 0.0, 2.0 * j as f32 - 50.0)),
                ));
            }
        }

        Scene {
            instances,
            models,
            textures: vec![],
        }
    }

    pub fn insert(&mut self, model: Model) {
        self.models.push(model)
    }
}
