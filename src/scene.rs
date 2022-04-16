use crate::block::Block;
use crate::vulkan::texture::Texture;
use crevice::std430::AsStd430;
use glam::{vec2, vec3, Vec2, Vec3};
use serde::Deserialize;
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
pub struct AtlasMeta {
    pub size: Size,
}

#[derive(Deserialize, Debug)]
pub struct TextureAtlas {
    pub frames: Vec<Frame>,
    pub meta: AtlasMeta,
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

#[derive(Default, Copy, Clone, Debug)]
pub struct QuadUVs {
    uv0: Vec2,
    uv1: Vec2,
    uv2: Vec2,
    uv3: Vec2,
}

impl QuadUVs {
    pub fn uv0(&self) -> Vec2 {
        self.uv0
    }
    pub fn uv1(&self) -> Vec2 {
        self.uv1
    }
    pub fn uv2(&self) -> Vec2 {
        self.uv2
    }
    pub fn uv3(&self) -> Vec2 {
        self.uv3
    }
}

impl QuadUVs {
    pub fn new(uv0: Vec2, uv1: Vec2, uv2: Vec2, uv3: Vec2) -> Self {
        QuadUVs { uv0, uv1, uv2, uv3 }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct BlockUVs {
    side: QuadUVs,
    top: QuadUVs,
    bot: QuadUVs,
}

impl BlockUVs {
    pub fn side(&self) -> &QuadUVs {
        &self.side
    }
    pub fn top(&self) -> &QuadUVs {
        &self.top
    }
    pub fn bot(&self) -> &QuadUVs {
        &self.bot
    }
}

impl BlockUVs {
    pub fn new(side: QuadUVs, top: QuadUVs, bot: QuadUVs) -> Self {
        BlockUVs { side, top, bot }
    }
}

pub struct Scene {
    textures: Vec<Texture>,
    uvs: HashMap<Block, BlockUVs>,
    texture_atlas: TextureAtlas,
    materials: Vec<Std430Material>,
}

fn get_quad_uv(texture: Option<String>, texture_atlas: &TextureAtlas) -> QuadUVs {
    if let Some(texture) = texture {
        let texture_coords = texture_atlas
            .frames
            .iter()
            .find(|frame| frame.filename == texture)
            .unwrap()
            .frame;

        let uv0 = vec2(
            texture_coords.x as f32,
            (texture_coords.y + texture_coords.h) as f32,
        );
        let uv1 = vec2(
            (texture_coords.x + texture_coords.w) as f32,
            (texture_coords.y + texture_coords.h) as f32,
        );
        let uv2 = vec2(
            (texture_coords.x + texture_coords.w) as f32,
            (texture_coords.y) as f32,
        );
        let uv3 = vec2(texture_coords.x as f32, (texture_coords.y) as f32);

        let size = vec2(
            texture_atlas.meta.size.w as f32,
            texture_atlas.meta.size.h as f32,
        );

        let uv0 = uv0 / size;
        let uv1 = uv1 / size;
        let uv2 = uv2 / size;
        let uv3 = uv3 / size;

        QuadUVs::new(uv0, uv1, uv2, uv3)
    } else {
        QuadUVs::default()
    }
}

fn get_uvs(block: Block, texture_atlas: &TextureAtlas) -> BlockUVs {
    let textures = block.texture_names();
    let side = get_quad_uv(textures.side(), texture_atlas);
    let top = get_quad_uv(textures.top(), texture_atlas);
    let bot = get_quad_uv(textures.bottom(), texture_atlas);

    BlockUVs::new(side, top, bot)
}

impl Scene {
    pub fn new() -> Self {
        log::debug!("loading textures");
        let textures = vec![Texture::load_texture("resources/textures/blocks.png")];

        let frames_file = File::open("resources/textures/blocks.json").unwrap();
        let texture_atlas: TextureAtlas = serde_json::from_reader(frames_file).unwrap();

        let mut uvs = HashMap::new();

        for block in Block::iter() {
            let block_uvs = get_uvs(block, &texture_atlas);
            uvs.insert(block, block_uvs);
        }

        log::debug!("loading materialas");
        let materials = vec![
            Material::new(vec3(1.0, 0.0, 0.0)).as_std430(),
            Material::new(vec3(0.0, 1.0, 0.0)).as_std430(),
            Material::new(vec3(0.0, 0.0, 1.0)).as_std430(),
        ];

        Scene {
            textures,
            uvs,
            texture_atlas,
            materials,
        }
    }

    pub fn get_block_uvs(&self, block: Block) -> BlockUVs {
        *self.uvs.get(&block).unwrap()
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
