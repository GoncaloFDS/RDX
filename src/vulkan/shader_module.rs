use crate::vulkan::device::Device;
use erupt::vk;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::{env, ptr};

pub struct ShaderModule {
    handle: vk::ShaderModule,
    device: Rc<Device>,
}

impl ShaderModule {
    pub fn new(device: Rc<Device>) -> Self {
        let shader: &[u8] = include_bytes!(env!("raster.spv"));
        // let path = env::current_dir()
        //     .unwrap()
        //     .join("assets")
        //     .join("shaders")
        //     .join(filename);
        // let mut shader_file =
        //     File::open(path).unwrap_or_else(|_| panic!("Failed to open shader file: {}", filename));
        // let mut bytes = vec![];
        // shader_file.read_to_end(&mut bytes).unwrap();
        //
        // let spv = erupt::utils::decode_spv(&bytes).unwrap();

        // let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(shader);

        let create_info = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ShaderModuleCreateFlags::empty(),
            code_size: shader.len(),
            p_code: shader.as_ptr() as *const u32,
        };

        let shader_module = unsafe { device.create_shader_module(&create_info, None).unwrap() };

        ShaderModule {
            handle: shader_module,
            device,
        }
    }

    pub fn create_shader_stage<'a>(
        &self,
        stages: vk::ShaderStageFlagBits,
        name: &'a CStr,
    ) -> vk::PipelineShaderStageCreateInfoBuilder<'a> {
        vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(stages)
            .module(self.handle)
            .name(name)
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(Some(self.handle), None);
        }
    }
}
