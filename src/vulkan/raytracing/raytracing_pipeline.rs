use crate::vulkan::device::Device;
use erupt::vk;

pub struct RaytracingPipeline {
    handle: vk::Pipeline,
}

impl RaytracingPipeline {
    pub fn new(
        device: &Device,
        pipeline_info: &[vk::RayTracingPipelineCreateInfoKHRBuilder],
        pipeline_cache: vk::PipelineCache,
    ) -> Self {
        let pipeline = unsafe {
            device
                .handle()
                .create_ray_tracing_pipelines_khr(
                    vk::DeferredOperationKHR::default(),
                    pipeline_cache,
                    pipeline_info,
                    None,
                )
                .unwrap()[0]
        };

        RaytracingPipeline { handle: pipeline }
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
