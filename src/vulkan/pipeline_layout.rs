use crate::vulkan::descriptor_set_layout::DescriptorSetLayout;
use crate::vulkan::device::Device;
use erupt::vk;
use std::rc::Rc;

pub struct PushConstantRanges {
    handle: vk::PushConstantRange,
}

impl PushConstantRanges {
    pub fn handle(&self) -> vk::PushConstantRange {
        self.handle
    }

    pub fn new(stages: vk::ShaderStageFlags, offset: u32, size: u32) -> Self {
        let handle = vk::PushConstantRangeBuilder::new()
            .stage_flags(stages)
            .offset(offset)
            .size(size)
            .build();

        PushConstantRanges { handle }
    }
}

pub struct PipelineLayout {
    handle: vk::PipelineLayout,
    device: Rc<Device>,
}

impl PipelineLayout {
    pub fn handle(&self) -> vk::PipelineLayout {
        self.handle
    }

    pub fn new(
        device: Rc<Device>,
        descriptor_set_layouts: &[&DescriptorSetLayout],
        push_constant_ranges: &[PushConstantRanges],
    ) -> Self {
        let descriptor_set_layouts = descriptor_set_layouts
            .iter()
            .map(|layout| layout.handle())
            .collect::<Vec<_>>();

        let push_constant_ranges = push_constant_ranges
            .iter()
            .map(|range| range.handle().into_builder())
            .collect::<Vec<_>>();

        let create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .set_layouts(&descriptor_set_layouts)
            .push_constant_ranges(&push_constant_ranges);
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
