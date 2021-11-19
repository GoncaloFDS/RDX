use crate::vulkan::descriptor_set_layout::DescriptorSetLayout;
use crate::vulkan::device::Device;
use erupt::vk;
use std::rc::Rc;

pub struct PipelineLayout {
    handle: vk::PipelineLayout,
    device: Rc<Device>,
}

impl PipelineLayout {
    pub fn handle(&self) -> vk::PipelineLayout {
        self.handle
    }

    pub fn new(device: Rc<Device>, descriptor_set_layout: &DescriptorSetLayout) -> Self {
        let descriptor_set_layouts = [descriptor_set_layout.handle()];

        let create_info =
            vk::PipelineLayoutCreateInfoBuilder::new().set_layouts(&descriptor_set_layouts);
        let pipeline_layout = unsafe { device.create_pipeline_layout(&create_info, None).unwrap() };

        PipelineLayout {
            handle: pipeline_layout,
            device,
        }
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline_layout(Some(self.handle), None);
        }
    }
}
