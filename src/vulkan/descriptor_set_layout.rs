use erupt::vk;

pub struct DescriptorSetLayout {
    handle: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn handle(&self) -> vk::DescriptorSetLayout {
        self.handle
    }
}
