use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::sampler::{Sampler, SamplerInfo};
use crate::vulkan::texture::Texture;
use erupt::vk;
use std::rc::Rc;

pub struct TextureImage {
    image: Image,
    image_view: ImageView,
    sampler: Sampler,
}

impl TextureImage {
    pub fn uninitialized(device: Rc<Device>) -> Self {
        TextureImage {
            image: Image::uninitialized(device.clone()),
            image_view: ImageView::uninitialized(device.clone()),
            sampler: Sampler::uninitialized(device.clone()),
        }
    }

    pub fn new(device: Rc<Device>, command_pool: &CommandPool, texture: &Texture) -> Self {
        let image_size = texture.width() * texture.height() * 4;

        let mut staging_buffer = Buffer::new(
            device.clone(),
            image_size as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
        );
        staging_buffer.allocate_memory(gpu_alloc::UsageFlags::HOST_ACCESS);
        staging_buffer.write_data(texture.pixels(), 0);

        let mut image = Image::new(
            device.clone(),
            vk::Extent2D {
                width: texture.width(),
                height: texture.height(),
            },
            vk::Format::R8G8B8A8_UNORM,
            None,
            None,
        );
        image.allocate_memory();

        let image_view = ImageView::new(
            device.clone(),
            image.handle(),
            image.format(),
            vk::ImageAspectFlags::COLOR,
        );
        let sampler = Sampler::new(device.clone(), &SamplerInfo::default());

        image.transition_image_layout(command_pool, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        image.copy_from(command_pool, &staging_buffer);
        image.transition_image_layout(command_pool, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

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
