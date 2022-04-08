use crate::vulkan::device::Device;
use erupt::vk;
use std::rc::Rc;

pub struct SamplerInfo {
    mag_filter: vk::Filter,
    min_filter: vk::Filter,
    address_mode_u: vk::SamplerAddressMode,
    address_mode_v: vk::SamplerAddressMode,
    address_mode_w: vk::SamplerAddressMode,
    anisotropy_enable: bool,
    max_anisotropy: u32,
    border_color: vk::BorderColor,
    unnormalized_coordinates: bool,
    compare_enable: bool,
    compare_op: vk::CompareOp,
    mipmap_mode: vk::SamplerMipmapMode,
    mip_lod_bias: f32,
    min_lod: f32,
    max_lod: f32,
}

impl Default for SamplerInfo {
    fn default() -> Self {
        SamplerInfo {
            mag_filter: vk::Filter::NEAREST,
            min_filter: vk::Filter::NEAREST,
            address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            anisotropy_enable: false,
            max_anisotropy: 16,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: false,
            compare_enable: false,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            mip_lod_bias: 0.0,
            min_lod: 0.0,
            max_lod: 0.0,
        }
    }
}

pub struct Sampler {
    handle: vk::Sampler,
}

impl Sampler {
    pub fn new(device: &Device, info: &SamplerInfo) -> Self {
        let create_info = vk::SamplerCreateInfoBuilder::new()
            .mag_filter(info.mag_filter)
            .min_filter(info.min_filter)
            .address_mode_u(info.address_mode_u)
            .address_mode_v(info.address_mode_v)
            .address_mode_w(info.address_mode_w)
            .anisotropy_enable(info.anisotropy_enable)
            .max_anisotropy(info.max_anisotropy as _)
            .border_color(info.border_color)
            .unnormalized_coordinates(info.unnormalized_coordinates)
            .compare_enable(info.compare_enable)
            .compare_op(info.compare_op)
            .mipmap_mode(info.mipmap_mode)
            .mip_lod_bias(info.mip_lod_bias)
            .min_lod(info.min_lod)
            .max_lod(info.max_lod);
        let sampler = unsafe { device.handle().create_sampler(&create_info, None).unwrap() };

        Sampler { handle: sampler }
    }

    pub fn handle(&self) -> vk::Sampler {
        self.handle
    }
}
