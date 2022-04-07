use crate::vulkan::device::Device;
use erupt::vk;
use std::ffi::CStr;

pub enum Shader {
    Raster,
    UI,
}

const RASTER_BYTES: &[u8] = include_bytes!(env!("raster.spv"));
const UI_BYTES: &[u8] = include_bytes!(env!("ui.spv"));

pub struct ShaderModule {
    handle: vk::ShaderModule,
}

impl ShaderModule {
    pub fn new(device: &Device, shader: Shader) -> Self {
        let bytes = match shader {
            Shader::Raster => RASTER_BYTES,
            Shader::UI => UI_BYTES,
        };
        let code = erupt::utils::decode_spv(bytes).unwrap();
        let module_info = vk::ShaderModuleCreateInfoBuilder::new().code(&code);
        let shader_module = unsafe {
            device
                .handle()
                .create_shader_module(&module_info, None)
                .unwrap()
        };

        ShaderModule {
            handle: shader_module,
        }
    }

    pub fn destroy(&self, device: &Device) {
        unsafe {
            device.handle().destroy_shader_module(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::ShaderModule {
        self.handle
    }

    pub fn shader_stage<'a>(
        &self,
        stages: vk::ShaderStageFlagBits,
        name: &'a str,
    ) -> vk::PipelineShaderStageCreateInfoBuilder<'a> {
        vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(stages)
            .module(self.handle)
            .name(CStr::from_bytes_with_nul(name.as_bytes()).unwrap())
    }
}
