use crate::vulkan::model::{Instance, Model};
use crate::vulkan::texture::Texture;
use crevice::std430::AsStd430;
use glam::{vec3, Mat4, Vec3, Vec4};

#[derive(AsStd430)]
pub struct Material {
    color: Vec3,
}

impl Material {
    pub fn new(color: Vec3) -> Self {
        Material { color }
    }
}

pub struct Scene {
    models: Vec<Model>,
    instances: Vec<Instance>,
    textures: Vec<Texture>,
    materials: Vec<Std430Material>,
}

impl Scene {
    pub fn new() -> Self {
        let models = vec![Model::cube()];

        let mut instances = vec![];
        for x in 0..32 {
            for y in -32..0 {
                for z in 0..32 {
                    instances.push(Instance::new(
                        x % 3,
                        0,
                        Mat4::from_translation(vec3(x as f32 - 50.0, y as f32, z as f32 - 50.0)),
                    ));
                }
            }
        }

        let textures = vec![
            Texture::load_texture("resources/textures/grass_side_carried.png"),
            Texture::load_texture("resources/textures/grass_carried.png"),
            Texture::load_texture("resources/textures/dirt.png"),
        ];

        let materials = vec![
            Material::new(vec3(1.0, 0.0, 0.0)).as_std430(),
            Material::new(vec3(0.0, 1.0, 0.0)).as_std430(),
            Material::new(vec3(0.0, 0.0, 1.0)).as_std430(),
        ];

        Scene {
            instances,
            models,
            textures,
            materials,
        }
    }
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

    pub fn materials(&self) -> &[Std430Material] {
        &self.materials
    }
}
