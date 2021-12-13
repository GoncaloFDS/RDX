#[derive(Copy, Clone, Debug, PartialEq)]
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
}

impl Default for Block {
    fn default() -> Self {
        Block::Empty
    }
}
