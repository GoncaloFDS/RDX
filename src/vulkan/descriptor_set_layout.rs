use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::device::Device;
use erupt::vk;

pub struct DescriptorSetLayout {
    handle: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(device: &Device, descriptor_bindings: &[DescriptorBinding]) -> Self {
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
                .handle()
                .create_descriptor_set_layout(&create_info, None)
                .unwrap()
        };

        DescriptorSetLayout {
            handle: descriptor_set_layout,
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device
                .handle()
                .destroy_descriptor_set_layout(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::DescriptorSetLayout {
        self.handle
    }
}
