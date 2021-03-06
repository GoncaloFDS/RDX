use crevice::std430::AsStd430;
use erupt::vk;
use glam::{Vec2, Vec3, Vec4};
use memoffset::offset_of;
use std::mem::size_of;

pub trait Vertex {
    fn binding_descriptions() -> Vec<vk::VertexInputBindingDescriptionBuilder<'static>>;
    fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescriptionBuilder<'static>>;
}

#[derive(Copy, Clone, Debug, AsStd430)]
pub struct ModelVertex {
    position: Vec3,
    uv: Vec2,
}

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
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Self, uv) as u32),
        ]
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct EguiVertex {
    pub position: Vec2,
    pub tex_coords: Vec2,
    pub color: Vec4,
}

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
