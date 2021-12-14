use crate::block::Block;
use crate::vulkan::model::{Mesh, DIRECTIONS};
use crate::vulkan::scene::Scene;
use bracket_noise::prelude::*;
use glam::{ivec3, vec3, IVec3, Mat4};

pub const CHUNK_SIZE: i32 = 16;
const CHUNK_HEIGHT: i32 = 100;
pub const MAP_SIZE: i32 = 16;
const WATER_THRESHOLD: i32 = 50;
const NOISE_SCALE: f32 = 0.03;

#[derive(Debug)]
pub struct Chunk {
    pub blocks: [Block; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize],
    pub world_position: IVec3,
}

impl Chunk {
    pub fn new(world_position: IVec3) -> Self {
        let mut chunk = Chunk {
            blocks: [Block::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT) as usize],
            world_position,
        };
        chunk.generate_blocks();
        chunk
    }

    pub fn compute_chunk_mesh(&self, scene: &Scene) -> Mesh {
        let mut mesh = Mesh::default();
        self.blocks.iter().enumerate().for_each(|(index, &block)| {
            self.update_mesh(
                scene,
                Chunk::get_position_from_index(index as i32),
                &mut mesh,
                block,
            )
        });
        mesh
    }

    pub fn get_position_from_index(index: i32) -> IVec3 {
        ivec3(
            index % CHUNK_SIZE as i32,
            (index / CHUNK_SIZE as i32) % CHUNK_HEIGHT as i32,
            index / (CHUNK_SIZE as i32 * CHUNK_HEIGHT as i32),
        )
    }

    pub fn get_index_from_position(local_position: IVec3) -> usize {
        (local_position.x
            + CHUNK_SIZE as i32 * local_position.y
            + CHUNK_SIZE as i32 * CHUNK_HEIGHT as i32 * local_position.z) as usize
    }

    pub fn set_block(&mut self, local_position: IVec3, block: Block) {
        assert!(local_position.x >= 0 && local_position.x < CHUNK_SIZE as i32);
        assert!(local_position.z >= 0 && local_position.z < CHUNK_SIZE as i32);
        assert!(local_position.y >= 0 && local_position.y < CHUNK_HEIGHT as i32);

        let index = Chunk::get_index_from_position(local_position);
        self.blocks[index] = block;
    }

    pub fn get_block_from_chunk_coordinates(&self, position: IVec3) -> Block {
        if position.x >= 0
            && position.x < CHUNK_SIZE as i32
            && position.z >= 0
            && position.z < CHUNK_SIZE as i32
            && position.y >= 0
            && position.y < CHUNK_HEIGHT as i32
        {
            let index = Chunk::get_index_from_position(position);
            self.blocks[index]
        } else {
            Block::Air
        }
    }

    pub fn to_chunk_coordinates(&self, position: IVec3) -> IVec3 {
        ivec3(
            position.x - self.world_position.x,
            position.y - self.world_position.y,
            position.z - self.world_position.z,
        )
    }

    fn update_mesh(&self, scene: &Scene, position: IVec3, mesh: &mut Mesh, block: Block) {
        if block == Block::Empty || block == Block::Air {
            return;
        }

        DIRECTIONS.iter().for_each(|dir| {
            let neighbour_block_coordinates = position + dir.get_vector();
            let neighbour_block =
                self.get_block_from_chunk_coordinates(neighbour_block_coordinates);

            if neighbour_block != Block::Empty && !neighbour_block.is_solid() {
                if block == Block::Water {
                    //todo!()
                } else {
                    let block_uvs = scene.get_block_uvs(block);
                    mesh.add_quad(*dir, position, block, block_uvs);
                }
            }
        });
    }

    pub fn generate_blocks(&mut self) {
        let seed = 255;
        let mut noise = FastNoise::seeded(seed);
        noise.set_noise_type(NoiseType::Perlin);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(1);
        noise.set_fractal_gain(0.2);
        noise.set_fractal_lacunarity(2.0);
        noise.set_frequency(0.9);

        for x in 0..(CHUNK_SIZE as i32) {
            for z in 0..(CHUNK_SIZE as i32) {
                let noise_value = noise.get_noise(
                    (self.world_position.x + x) as f32 * NOISE_SCALE,
                    (self.world_position.z + z) as f32 * NOISE_SCALE,
                );
                let noise_value = map(noise_value, -1.0, 1.0, 0.0, 1.0);
                let ground_position = (noise_value * CHUNK_HEIGHT as f32).round() as i32;

                for y in 0..(CHUNK_HEIGHT as i32) {
                    let block;
                    if y > ground_position {
                        block = Block::Air;
                    } else if y == ground_position {
                        block = Block::Grass;
                    } else {
                        block = Block::Dirt;
                    }

                    self.set_block(ivec3(x, y, z), block);
                }
            }
        }
    }

    pub fn transform(&self) -> Mat4 {
        Mat4::from_translation(vec3(
            self.world_position.x as f32,
            self.world_position.y as f32,
            self.world_position.z as f32,
        ))
    }
}

pub fn map(value: f32, min1: f32, max1: f32, min2: f32, max2: f32) -> f32 {
    min2 + (value - min1) * (max2 - min2) / (max1 - min1)
}
