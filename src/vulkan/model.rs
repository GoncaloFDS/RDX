use crate::block::{Block, Sides};
use crate::vulkan::scene::BlockUVs;
use crate::vulkan::vertex::{ModelVertex, Std430ModelVertex};
use crevice::std430::AsStd430;
use glam::*;

pub const DIRECTIONS: [Direction; 6] = [
    Direction::Back,
    Direction::Forwards,
    Direction::Left,
    Direction::Right,
    Direction::Up,
    Direction::Down,
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Back,
    Forwards,
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn get_vector(&self) -> IVec3 {
        match *self {
            Direction::Back => -IVec3::Z,
            Direction::Forwards => IVec3::Z,
            Direction::Left => -IVec3::X,
            Direction::Right => IVec3::X,
            Direction::Up => IVec3::Y,
            Direction::Down => -IVec3::Y,
        }
    }
}

pub struct Instance {
    id: u32,
    blas_id: u32,
    transform: Mat4,
}

impl Instance {
    pub fn new(id: u32, blas_id: u32, transform: Mat4) -> Self {
        Instance {
            id,
            blas_id,
            transform,
        }
    }
}

impl Instance {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn blas_id(&self) -> u32 {
        self.blas_id
    }

    pub fn transform(&self) -> Mat4 {
        self.transform
    }
}

#[derive(Default)]
pub struct Mesh {
    vertices: Vec<Std430ModelVertex>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn vertices(&self) -> &[Std430ModelVertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}

impl Mesh {
    pub fn add_vertex(&mut self, position: Vec3) {
        self.vertices
            .push(ModelVertex::new(position, Vec2::ZERO, 0).as_std430())
    }

    pub fn add_quad(
        &mut self,
        direction: Direction,
        position: IVec3,
        block: Block,
        block_uvs: BlockUVs,
    ) {
        self.add_quad_vertices(direction, position, block, block_uvs);
        self.compute_quad_indices();
    }

    fn add_quad_vertices(
        &mut self,
        direction: Direction,
        position: IVec3,
        block: Block,
        block_uvs: BlockUVs,
    ) {
        let x = position.x as f32;
        let y = position.y as f32;
        let z = position.z as f32;

        let sides = block.side_count();

        let side_uv = block_uvs.side();
        let mut top_uv = block_uvs.top();
        let mut bot_uv = block_uvs.bot();

        if sides == Sides::AllEqual {
            top_uv = side_uv;
            bot_uv = side_uv;
        }

        let quad = match direction {
            Direction::Back => [
                ModelVertex::new(vec3(x - 0.5, y - 0.5, z - 0.5), side_uv.uv0(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y - 0.5, z - 0.5), side_uv.uv1(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y + 0.5, z - 0.5), side_uv.uv2(), 0).as_std430(),
                ModelVertex::new(vec3(x - 0.5, y + 0.5, z - 0.5), side_uv.uv3(), 0).as_std430(),
            ],
            Direction::Forwards => [
                ModelVertex::new(vec3(x + 0.5, y - 0.5, z + 0.5), side_uv.uv0(), 0).as_std430(),
                ModelVertex::new(vec3(x - 0.5, y - 0.5, z + 0.5), side_uv.uv1(), 0).as_std430(),
                ModelVertex::new(vec3(x - 0.5, y + 0.5, z + 0.5), side_uv.uv2(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y + 0.5, z + 0.5), side_uv.uv3(), 0).as_std430(),
            ],
            Direction::Left => [
                ModelVertex::new(vec3(x - 0.5, y - 0.5, z - 0.5), side_uv.uv0(), 0).as_std430(),
                ModelVertex::new(vec3(x - 0.5, y - 0.5, z + 0.5), side_uv.uv1(), 0).as_std430(),
                ModelVertex::new(vec3(x - 0.5, y + 0.5, z + 0.5), side_uv.uv2(), 0).as_std430(),
                ModelVertex::new(vec3(x - 0.5, y + 0.5, z - 0.5), side_uv.uv3(), 0).as_std430(),
            ],
            Direction::Right => [
                ModelVertex::new(vec3(x + 0.5, y - 0.5, z + 0.5), side_uv.uv0(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y - 0.5, z - 0.5), side_uv.uv1(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y + 0.5, z - 0.5), side_uv.uv2(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y + 0.5, z + 0.5), side_uv.uv3(), 0).as_std430(),
            ],
            Direction::Up => [
                ModelVertex::new(vec3(x - 0.5, y + 0.5, z - 0.5), top_uv.uv0(), 0).as_std430(),
                ModelVertex::new(vec3(x - 0.5, y + 0.5, z + 0.5), top_uv.uv1(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y + 0.5, z + 0.5), top_uv.uv2(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y + 0.5, z - 0.5), top_uv.uv3(), 0).as_std430(),
            ],
            Direction::Down => [
                ModelVertex::new(vec3(x - 0.5, y - 0.5, z + 0.5), bot_uv.uv0(), 0).as_std430(),
                ModelVertex::new(vec3(x - 0.5, y - 0.5, z - 0.5), bot_uv.uv1(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y - 0.5, z - 0.5), bot_uv.uv2(), 0).as_std430(),
                ModelVertex::new(vec3(x + 0.5, y - 0.5, z + 0.5), bot_uv.uv3(), 0).as_std430(),
            ],
        };
        self.vertices.extend_from_slice(&quad);
    }

    pub fn compute_quad_indices(&mut self) {
        self.indices.push(self.vertices.len() as u32 - 4);
        self.indices.push(self.vertices.len() as u32 - 3);
        self.indices.push(self.vertices.len() as u32 - 2);

        self.indices.push(self.vertices.len() as u32 - 4);
        self.indices.push(self.vertices.len() as u32 - 2);
        self.indices.push(self.vertices.len() as u32 - 1);
    }
}
