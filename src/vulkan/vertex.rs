use crate::vulkan::buffer::Buffer;
use bytemuck::{Pod, Zeroable};
use crevice::std430::{AsStd430, Std430};
use erupt::vk;
use glam::{Vec2, Vec3, Vec4};
use memoffset::offset_of;
use std::mem::size_of;
use std::sync::Arc;

pub trait Vertex {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescriptionBuilder<'static>>;
    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescriptionBuilder<'static>>;
}

#[derive(Copy, Clone, Debug, AsStd430)]
pub struct ModelVertex {
    position: Vec3,
    uv: Vec2,
}

unsafe impl Zeroable for ModelVertex {}
unsafe impl Pod for ModelVertex {}

impl ModelVertex {
    pub fn new(position: Vec3, uv: Vec2) -> Self {
        ModelVertex { position, uv }
    }
}

impl Vertex for ModelVertex {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescriptionBuilder<'static>> {
        vec![vk::VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)]
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescriptionBuilder<'static>> {
        vec![
            vk::VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, position) as u32),
            vk::VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, uv) as u32),
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EguiVertex {
    pub position: Vec2,
    pub tex_coords: Vec2,
    pub color: Vec4,
}

unsafe impl Zeroable for EguiVertex {}

unsafe impl Pod for EguiVertex {}

impl EguiVertex {
    pub fn new(position: Vec2, tex_coords: Vec2, color: Vec4) -> Self {
        EguiVertex {
            position,
            tex_coords,
            color,
        }
    }
}

impl Vertex for EguiVertex {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescriptionBuilder<'static>> {
        vec![vk::VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)]
    }

    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescriptionBuilder<'static>> {
        vec![
            vk::VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, position) as u32),
            vk::VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Self, tex_coords) as u32),
            vk::VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .offset(offset_of!(Self, color) as u32),
        ]
    }
}

pub struct VertexBuffer {
    buffer: Arc<Buffer>,
    offset: u64,
    element_count: u32,
}

impl VertexBuffer {
    pub fn new(buffer: Arc<Buffer>, offset: u64, element_count: u32) -> Self {
        VertexBuffer {
            buffer,
            offset,
            element_count,
        }
    }
}

impl VertexBuffer {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }
    pub fn element_count(&self) -> u32 {
        self.element_count
    }
}

pub struct IndexBuffer {
    buffer: Arc<Buffer>,
    offset: u64,
    element_count: u32,
}

impl IndexBuffer {
    pub fn new(buffer: Arc<Buffer>, offset: u64, element_count: u32) -> Self {
        IndexBuffer {
            buffer,
            offset,
            element_count,
        }
    }
}

impl IndexBuffer {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
    pub fn offset(&self) -> u64 {
        self.offset
    }
    pub fn element_count(&self) -> u32 {
        self.element_count
    }
}
