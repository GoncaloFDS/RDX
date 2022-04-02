use crate::vulkan::device::Device;
use erupt::vk;

pub struct GraphicsPipeline {
    handle: vk::Pipeline,
}

impl GraphicsPipeline {
    pub fn new(
        device: &Device,
        pipeline_info: &[vk::GraphicsPipelineCreateInfoBuilder],
        pipeline_cache: vk::PipelineCache,
    ) -> Self {
        let pipeline = unsafe {
            device
                .handle()
                .create_graphics_pipelines(pipeline_cache, pipeline_info, None)
                .unwrap()[0]
        };

        GraphicsPipeline { handle: pipeline }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.handle().destroy_pipeline(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }
}
