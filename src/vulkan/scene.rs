use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::model::{Instance, Model};
use crate::vulkan::texture::Texture;
use glam::{vec3, Mat4, Vec3, Vec4};
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct Material {
    color: Vec4,
}

impl Material {
    pub fn new(color: Vec3) -> Self {
        Material {
            color: color.extend(1.0),
        }
    }
}

pub struct Scene {
    instances: Vec<Instance>,
    models: Vec<Model>,
    textures: Vec<Texture>,
    materials: Vec<Material>,
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

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }
}

impl Scene {
    pub fn new() -> Self {
        let models = vec![Model::cube()];
        let mut instances = vec![];

        for i in 0..100 {
            for j in 0..100 {
                instances.push(Instance::new(
                    i % 3,
                    0,
                    Mat4::from_translation(vec3(2.0 * i as f32 - 50.0, 0.0, 2.0 * j as f32 - 50.0)),
                ));
            }
        }

        let materials = vec![
            Material::new(vec3(1.0, 0.0, 0.0)),
            Material::new(vec3(0.0, 1.0, 0.0)),
            Material::new(vec3(0.0, 0.0, 1.0)),
        ];

        Scene {
            instances,
            models,
            textures: vec![],
            materials,
        }
    }

    pub fn insert(&mut self, model: Model) {
        self.models.push(model)
    }
}
