use crate::block::Block;
use crate::vulkan::model::{Mesh, DIRECTIONS};
use glam::{ivec3, vec3, IVec3, Mat4};
use noise::NoiseFn;

pub const CHUNK_SIZE: i32 = 16;
const CHUNK_HEIGHT: i32 = 100;
pub const MAP_SIZE: i32 = 16;
const WATER_THRESHOLD: i32 = 50;
const NOISE_SCALE: f64 = 0.01;

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

    pub fn compute_chunk_mesh(&self) -> Mesh {
        let mut mesh = Mesh::default();
        self.blocks.iter().enumerate().for_each(|(index, &block)| {
            self.update_mesh(
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

    fn update_mesh(&self, position: IVec3, mesh: &mut Mesh, block: Block) {
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
                    mesh.add_quad(*dir, position, block);
                }
            }
        });
    }

    pub fn generate_blocks(&mut self) {
        let perlin = noise::Perlin::new();
        for x in 0..(CHUNK_SIZE as i32) {
            for z in 0..(CHUNK_SIZE as i32) {
                let noise_value = perlin.get([
                    (self.world_position.x + x) as f64 * NOISE_SCALE,
                    (self.world_position.z + z) as f64 * NOISE_SCALE,
                ]);
                let ground_position = (noise_value * CHUNK_HEIGHT as f64).round() as i32;

                for y in 0..(CHUNK_HEIGHT as i32) {
                    let block;
                    if y > ground_position {
                        if y < WATER_THRESHOLD {
                            block = Block::Water;
                        } else {
                            block = Block::Air;
                        }
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
