use crate::vulkan::model::Model;
use crate::vulkan::texture::Texture;

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
        let models = vec![Model::triangle()];
        Scene {
            models,
            textures: vec![],
        }
    }
}
