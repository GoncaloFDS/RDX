use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::sampler::{Sampler, SamplerInfo};
use crate::vulkan::texture::Texture;
use erupt::vk;

pub struct TextureImage {
    image: Image,
    sampler: Sampler,
}

impl TextureImage {
    pub fn new(device: &mut Device, texture: &Texture) -> Self {
        let mut staging_buffer =
            Buffer::with_data(device, texture.pixels(), vk::BufferUsageFlags::TRANSFER_SRC);

        let mut image = Image::new(
            device,
            vk::Extent2D {
                width: texture.width(),
                height: texture.height(),
            },
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::ImageAspectFlags::COLOR,
        );

        let sampler = Sampler::new(device, &SamplerInfo::default());

        let command_pool = device.command_pool();
        image.transition_image_layout(device, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        image.copy_from(device, &staging_buffer);
        image.transition_image_layout(device, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

        staging_buffer.destroy(device);
        TextureImage { image, sampler }
    }

    pub fn destroy(&mut self, device: &mut Device) {
        self.sampler.destroy(device);
        self.image.destroy(device);
    }
}

impl TextureImage {
    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn image_view(&self) -> vk::ImageView {
        self.image.view()
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }
}
