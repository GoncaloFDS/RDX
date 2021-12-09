use image::GenericImageView;
use std::path::Path;

#[derive(Default)]
pub struct Texture {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Texture {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Texture {
            width,
            height,
            pixels,
        }
    }

    pub fn load_texture<P: AsRef<Path>>(path: P) -> Self {
        let img = image::open(path).unwrap();
        Texture {
            width: img.width(),
            height: img.height(),
            pixels: Vec::from(img.as_bytes()),
        }
    }
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
}
