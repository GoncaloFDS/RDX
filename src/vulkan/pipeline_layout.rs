use crate::vulkan::descriptor_set_layout::DescriptorSetLayout;
use crate::vulkan::device::Device;
use crate::vulkan::push_constants::PushConstantRanges;
use erupt::{vk, SmallVec};

pub struct PipelineLayout {
    handle: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn new(
        device: &Device,
        descriptor_set_layouts: &[&DescriptorSetLayout],
        push_constant_ranges: &[&PushConstantRanges],
    ) -> Self {
        let descriptor_set_layouts = descriptor_set_layouts
            .iter()
            .map(|layout| layout.handle())
            .collect::<SmallVec<_>>();

        let push_constant_ranges = push_constant_ranges
            .iter()
            .map(|range| range.handle().into_builder())
            .collect::<SmallVec<_>>();

        let create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);

        let pipeline_layout = unsafe {
            device
                .handle()
                .create_pipeline_layout(&create_info, None)
                .unwrap()
        };

        PipelineLayout {
            handle: pipeline_layout,
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.handle().destroy_pipeline_layout(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::PipelineLayout {
        self.handle
    }
}
