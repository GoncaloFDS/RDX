use erupt::vk;
use glam::{Vec2, Vec3};
use memoffset::offset_of;
use std::mem::size_of;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    // pub normal: Vec3,
    // pub tex_coords: Vec2,
    // pub material_index: i32,
}

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
