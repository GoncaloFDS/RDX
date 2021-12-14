use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, EnumIter)]
pub enum Block {
    Empty,
    Grass,
    Dirt,
    Stone,
    Water,
    Air,
}

impl Block {
    pub fn is_solid(self) -> bool {
        match self {
            Block::Water => true,
            Block::Air => false,
            _ => true,
        }
    }

    pub fn texture_names(self) -> BlockTextureNames {
        match self {
            Block::Empty => BlockTextureNames::default(),
            Block::Grass => BlockTextureNames::new(
                Some("grass_carried.png".into()),
                Some("grass_side_carried.png".into()),
                Some("dirt.png".into()),
            ),
            Block::Dirt => BlockTextureNames::new(Some("dirt.png".into()), None, None),
            Block::Stone => BlockTextureNames::default(),
            Block::Water => BlockTextureNames::default(),
            Block::Air => BlockTextureNames::default(),
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Block::Empty
    }
}

#[derive(Default, Debug)]
pub struct BlockTextureNames {
    side: Option<String>,
    top: Option<String>,
    bottom: Option<String>,
}

impl BlockTextureNames {
    pub fn new(side: Option<String>, top: Option<String>, bottom: Option<String>) -> Self {
        BlockTextureNames { side, top, bottom }
    }
}

impl BlockTextureNames {
    pub fn side(&self) -> Option<String> {
        self.side.clone()
    }
    pub fn top(&self) -> Option<String> {
        self.top.clone()
    }
    pub fn bottom(&self) -> Option<String> {
        self.bottom.clone()
    }
}

#[derive(Debug)]
pub struct BlockTextures {
    pub side: Option<u32>,
    pub top: Option<u32>,
    pub bottom: Option<u32>,
}

impl BlockTextures {
    pub fn new(side: Option<u32>, top: Option<u32>, bottom: Option<u32>) -> Self {
        BlockTextures { side, top, bottom }
    }
}
