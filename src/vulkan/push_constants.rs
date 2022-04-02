use erupt::vk;

pub struct PushConstantRanges {
    handle: vk::PushConstantRange,
}

impl PushConstantRanges {
    pub fn handle(&self) -> vk::PushConstantRange {
        self.handle
    }
}
