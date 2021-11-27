use crate::vulkan::device::Device;
use erupt::vk;
use std::rc::Rc;

pub struct ImageView {
    handle: vk::ImageView,
    device: Rc<Device>,
}

impl ImageView {
    pub fn handle(&self) -> vk::ImageView {
        self.handle
    }

    pub fn uninitialized(device: Rc<Device>) -> Self {
        ImageView {
            handle: Default::default(),
            device,
        }
    }

    pub fn new(
        device: Rc<Device>,
        image: vk::Image,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
    ) -> Self {
        let create_info = vk::ImageViewCreateInfoBuilder::new()
            .image(image)
            .view_type(vk::ImageViewType::_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: aspect_flags,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let image_view = unsafe { device.create_image_view(&create_info, None).unwrap() };

        ImageView {
            handle: image_view,
            device,
        }
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(Some(self.handle), None);
        }
    }
}
