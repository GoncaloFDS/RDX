use crate::renderers::Renderer;
use crate::user_interface::UserInterface;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use erupt::vk;
use std::slice;

pub struct Clear {
    render_area: vk::Rect2D,
}

impl Clear {
    pub fn new(device: &Device) -> Self {
        let render_area = vk::Rect2D {
            offset: Default::default(),
            extent: device.swapchain().extent(),
        };
        Clear { render_area }
    }
}

impl Renderer for Clear {
    fn fill_command_buffer(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
        current_image: usize,
    ) {
        puffin::profile_function!();
        let color_attachment = vk::RenderingAttachmentInfoBuilder::new()
            .image_view(*device.swapchain_image_view(current_image))
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.4, 0.3, 0.2, 1.0],
                },
            })
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .resolve_image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let rendering_info = vk::RenderingInfoBuilder::new()
            .color_attachments(slice::from_ref(&color_attachment))
            .layer_count(1)
            .render_area(self.render_area);

        command_buffer.set_scissor(device, 0, &[self.render_area.into_builder()]);

        let extent = self.render_area.extent;
        let viewports = vk::ViewportBuilder::new()
            .height(extent.height as f32)
            .width(extent.width as f32)
            .max_depth(1.0);
        command_buffer.set_viewport(device, 0, &[viewports]);

        command_buffer.begin_rendering(device, &rendering_info);
    }

    fn update(&mut self, device: &mut Device, _ui: &mut UserInterface) {
        self.render_area = vk::Rect2D {
            offset: Default::default(),
            extent: device.swapchain().extent(),
        };
    }

    fn destroy(&mut self, _device: &mut Device) {}
}
