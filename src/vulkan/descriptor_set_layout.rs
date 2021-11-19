use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::device::Device;
use erupt::vk;
use std::rc::Rc;

pub struct DescriptorSetLayout {
    handle: vk::DescriptorSetLayout,
    device: Rc<Device>,
}

impl DescriptorSetLayout {
    pub fn handle(&self) -> vk::DescriptorSetLayout {
        self.handle
    }

    pub fn new(device: Rc<Device>, descriptor_bindings: &[DescriptorBinding]) -> Self {
        let layout_bindings = descriptor_bindings
            .iter()
            .map(|binding| {
                vk::DescriptorSetLayoutBindingBuilder::new()
                    .binding(binding.binding)
                    .descriptor_count(binding.descriptor_count)
                    .descriptor_type(binding.descriptor_type)
                    .stage_flags(binding.stages)
            })
            .collect::<Vec<_>>();

        let create_info =
            vk::DescriptorSetLayoutCreateInfoBuilder::new().bindings(&layout_bindings);

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&create_info, None)
                .unwrap()
        };

        DescriptorSetLayout {
            handle: descriptor_set_layout,
            device,
        }
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_descriptor_set_layout(Some(self.handle), None);
        }
    }
}
