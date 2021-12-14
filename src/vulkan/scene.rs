use crate::block::{Block, BlockTextures};
use crate::vulkan::texture::Texture;
use crevice::std430::AsStd430;
use glam::{vec3, Vec3};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use strum::IntoEnumIterator;

#[derive(Deserialize, Copy, Clone, Debug, Default)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Deserialize, Debug)]
pub struct Size {
    pub w: i32,
    pub h: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub filename: String,
    pub frame: Coords,
    pub rotated: bool,
    pub trimmed: bool,
    pub sprite_source_size: Coords,
    pub source_size: Size,
}

#[derive(Deserialize, Debug)]
pub struct TextureAtlas {
    pub frames: Vec<Frame>,
}

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
    map: HashMap<u32, Coords>,
    other: HashMap<Block, BlockTextures>,
    texture_atlas: TextureAtlas,
    materials: Vec<Std430Material>,
}

impl Scene {
    pub fn new() -> Self {
        let textures = vec![Texture::load_texture("resources/textures/blocks.png")];

        let mut frames_file = File::open("resources/textures/blocks.json").unwrap();
        let texture_atlas: TextureAtlas = serde_json::from_reader(frames_file).unwrap();

        let mut map = HashMap::new();
        let mut other = HashMap::new();

        for block in Block::iter() {
            let names = block.texture_names();
            let side = names.side().map(|side_texture| {
                texture_atlas
                    .frames
                    .iter()
                    .enumerate()
                    .find(|(_, frame)| frame.filename == side_texture)
                    .map(|(id, frame)| (id, frame.frame))
                    .unwrap()
            });
            if let Some((id, tex)) = side {
                map.insert(id as u32, tex);
                log::debug!("{:?}: id : {}, tex: {:?}", block, id, tex);
            }

            let top = names.top().map(|side_texture| {
                texture_atlas
                    .frames
                    .iter()
                    .enumerate()
                    .find(|(_, frame)| frame.filename == side_texture)
                    .map(|(id, frame)| (id, frame.frame))
                    .unwrap()
            });
            if let Some((id, tex)) = top {
                map.insert(id as u32, tex);
                log::debug!("{:?}: id : {}, tex: {:?}", block, id, tex);
            }

            let bottom = names.bottom().map(|side_texture| {
                texture_atlas
                    .frames
                    .iter()
                    .enumerate()
                    .find(|(_, frame)| frame.filename == side_texture)
                    .map(|(id, frame)| (id, frame.frame))
                    .unwrap()
            });
            if let Some((id, tex)) = bottom {
                map.insert(id as u32, tex);
                log::debug!("{:?}: id : {}, tex: {:?}", block, id, tex);
            }

            other.insert(
                block,
                BlockTextures::new(
                    side.map(|a| a.0 as u32),
                    top.map(|a| a.0 as u32),
                    bottom.map(|a| a.0 as u32),
                ),
            );
        }

        log::debug!("other: {:?}", other);

        let materials = vec![
            Material::new(vec3(1.0, 0.0, 0.0)).as_std430(),
            Material::new(vec3(0.0, 1.0, 0.0)).as_std430(),
            Material::new(vec3(0.0, 0.0, 1.0)).as_std430(),
        ];

        Scene {
            textures,
            map,
            other,
            texture_atlas,
            materials,
        }
    }

    pub fn get_texture_coords(&self, block: Block) -> Coords {
        let block_textures = self.other.get(&block).unwrap();
        if let Some(side_id) = block_textures.side {
            self.map.get(&side_id).unwrap().clone()
        } else {
            Coords::default()
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
