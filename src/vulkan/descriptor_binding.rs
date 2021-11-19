use erupt::vk;

pub struct DescriptorBinding {
    pub binding: u32,
    pub descriptor_count: u32,
    pub descriptor_type: vk::DescriptorType,
    pub stages: vk::ShaderStageFlags,
}

impl DescriptorBinding {
    pub fn new(
        binding: u32,
        descriptor_count: u32,
        descriptor_type: vk::DescriptorType,
        stage: vk::ShaderStageFlags,
    ) -> Self {
        DescriptorBinding {
            binding,
            descriptor_count,
            descriptor_type,
            stages: stage,
        }
    }
}
