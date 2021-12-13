use crate::vulkan::texture::Texture;
use crevice::std430::AsStd430;
use glam::{vec3, Vec3};

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
    textures: Vec<Texture>,
    materials: Vec<Std430Material>,
}

impl Scene {
    pub fn new() -> Self {
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
            textures,
            materials,
        }
    }
}

impl Scene {
    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }

    pub fn materials(&self) -> &[Std430Material] {
        &self.materials
    }
}
