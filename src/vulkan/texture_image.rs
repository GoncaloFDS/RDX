use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::sampler::{Sampler, SamplerInfo};
use crate::vulkan::texture::Texture;
use erupt::vk;

pub struct TextureImage {
    image: Image,
    image_view: ImageView,
    sampler: Sampler,
}

impl TextureImage {
    pub fn new(device: &mut Device, texture: &Texture) -> Self {
        let staging_buffer =
            Buffer::with_data(device, texture.pixels(), vk::BufferUsageFlags::TRANSFER_SRC);

        let mut image = Image::new(
            device,
            vk::Extent2D {
                width: texture.width(),
                height: texture.height(),
            },
            vk::Format::R8G8B8A8_UNORM,
            None,
            None,
        );
        image.allocate_memory(device);

        let image_view = ImageView::new(
            device,
            image.handle(),
            image.format(),
            vk::ImageAspectFlags::COLOR,
        );
        let sampler = Sampler::new(device, &SamplerInfo::default());

        let command_pool = device.command_pool();
        image.transition_image_layout(device, command_pool, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        image.copy_from(device, command_pool, &staging_buffer);
        image.transition_image_layout(
            device,
            command_pool,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );

        TextureImage {
            image,
            image_view,
            sampler,
        }
    }
}

impl TextureImage {
    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn image_view(&self) -> &ImageView {
        &self.image_view
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }
}
