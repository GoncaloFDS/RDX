pub struct Texture {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Texture {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Texture {
            width,
            height,
            pixels,
        }
    }
}
