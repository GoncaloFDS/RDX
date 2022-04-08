use erupt::vk;

pub struct PushConstantRanges {
    handle: vk::PushConstantRange,
}

impl PushConstantRanges {
    pub fn new(stages: vk::ShaderStageFlags, offset: u32, size: u32) -> Self {
        let handle = vk::PushConstantRangeBuilder::new()
            .stage_flags(stages)
            .offset(offset)
            .size(size)
            .build();

        PushConstantRanges { handle }
    }

    pub fn handle(&self) -> vk::PushConstantRange {
        self.handle
    }
}
