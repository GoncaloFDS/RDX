use bytemuck::{Pod, Zeroable};
use erupt::vk;
use glam::{Vec2, Vec3, Vec4};
use memoffset::offset_of;
use std::mem::size_of;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    // pub normal: Vec3,
    // pub tex_coords: Vec2,
    // pub material_index: i32,
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, tex_coords: Vec2, material_index: i32) -> Self {
        Vertex {
            position,
            // normal,
            // tex_coords,
            // material_index,
        }
    }

    pub fn binding_descriptions() -> [vk::VertexInputBindingDescriptionBuilder<'static>; 1] {
        [vk::VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)]
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescriptionBuilder<'static>; 1] {
        [
            vk::VertexInputAttributeDescriptionBuilder::new()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, position) as u32),
            // vk::VertexInputAttributeDescriptionBuilder::new()
            //     .binding(0)
            //     .location(1)
            //     .format(vk::Format::R32G32B32_SFLOAT)
            //     .offset(offset_of!(Self, normal) as u32),
            // vk::VertexInputAttributeDescriptionBuilder::new()
            //     .binding(0)
            //     .location(2)
            //     .format(vk::Format::R32G32_SFLOAT)
            //     .offset(offset_of!(Self, tex_coords) as u32),
            // vk::VertexInputAttributeDescriptionBuilder::new()
            //     .binding(0)
            //     .location(3)
            //     .format(vk::Format::R32_SINT)
            //     .offset(offset_of!(Self, material_index) as u32),
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

    pub fn binding_descriptions() -> [vk::VertexInputBindingDescriptionBuilder<'static>; 1] {
        [vk::VertexInputBindingDescriptionBuilder::new()
            .binding(0)
            .stride(size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)]
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescriptionBuilder<'static>; 3] {
        [
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
