use crate::vulkan::depth_buffer::DepthBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::swapchain::Swapchain;
use erupt::vk;
use std::rc::Rc;

pub struct RenderPass {
    handle: vk::RenderPass,
    device: Rc<Device>,
}

impl RenderPass {
    pub fn handle(&self) -> vk::RenderPass {
        self.handle
    }

    pub fn uninitialized(device: Rc<Device>) -> Self {
        RenderPass {
            handle: Default::default(),
            device,
        }
    }

    pub fn new(
        device: Rc<Device>,
        swapchain: &Swapchain,
        depth_buffer: &DepthBuffer,
        color_buffer_load_op: vk::AttachmentLoadOp,
        depth_buffer_load_op: vk::AttachmentLoadOp,
    ) -> Self {
        let attachments = [
            // Color
            vk::AttachmentDescriptionBuilder::new()
                .format(swapchain.format())
                .samples(vk::SampleCountFlagBits::_1)
                .load_op(color_buffer_load_op)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(if color_buffer_load_op == vk::AttachmentLoadOp::CLEAR {
                    vk::ImageLayout::UNDEFINED
                } else {
                    vk::ImageLayout::PRESENT_SRC_KHR
                })
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR),
            // Depth
            vk::AttachmentDescriptionBuilder::new()
                .format(depth_buffer.get_format())
                .samples(vk::SampleCountFlagBits::_1)
                .load_op(depth_buffer_load_op)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(if depth_buffer_load_op == vk::AttachmentLoadOp::CLEAR {
                    vk::ImageLayout::UNDEFINED
                } else {
                    vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
                })
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        ];

        let color_attachments = [vk::AttachmentReferenceBuilder::new()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];

        let depth_attachment = vk::AttachmentReferenceBuilder::new()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let subpasses = [vk::SubpassDescriptionBuilder::new()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .depth_stencil_attachment(&depth_attachment)];

        let subpass_dependencies = [vk::SubpassDependencyBuilder::new()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            )];

        let create_info = vk::RenderPassCreateInfoBuilder::new()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe { device.create_render_pass(&create_info, None).unwrap() };
        RenderPass {
            handle: render_pass,
            device,
        }
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(Some(self.handle), None);
        }
    }
}
