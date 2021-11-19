use crate::vulkan::depth_buffer::DepthBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::render_pass::RenderPass;
use crate::vulkan::swapchain::Swapchain;
use erupt::vk;
use std::rc::Rc;

pub struct Framebuffer {
    handle: vk::Framebuffer,
    device: Rc<Device>,
}

impl Framebuffer {
    pub fn new(
        device: Rc<Device>,
        image_view: &ImageView,
        render_pass: &RenderPass,
        swapchain: &Swapchain,
        depth_buffer: &DepthBuffer,
    ) -> Self {
        let attachments = [image_view.handle(), depth_buffer.image_view().handle()];
        let create_info = vk::FramebufferCreateInfoBuilder::new()
            .render_pass(**render_pass)
            .attachments(&attachments)
            .width(swapchain.extent().width)
            .height(swapchain.extent().height)
            .layers(1);

        let framebuffer = unsafe { device.create_framebuffer(&create_info, None).unwrap() };

        Framebuffer {
            handle: framebuffer,
            device,
        }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_framebuffer(Some(self.handle), None);
        }
    }
}
