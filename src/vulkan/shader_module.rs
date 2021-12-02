use crate::vulkan::device::Device;
use erupt::vk;
use std::ffi::CStr;
use std::rc::Rc;
use std::{env, ptr};

const RASTER: &[u8] = include_bytes!(env!("raster.spv"));
const UI: &[u8] = include_bytes!(env!("ui.spv"));
const RAYTRACING: &[u8] = include_bytes!(env!("raytracing.spv"));

pub struct ShaderModule {
    handle: vk::ShaderModule,
    device: Rc<Device>,
}

impl ShaderModule {
    pub fn new(device: Rc<Device>, shader: &str) -> Self {
        let shader = match shader {
            "raster" => RASTER,
            "ui" => UI,
            "raytracing" => RAYTRACING,
            _ => panic!(),
        };

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
        name: &'a str,
    ) -> vk::PipelineShaderStageCreateInfoBuilder<'a> {
        vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(stages)
            .module(self.handle)
            .name(CStr::from_bytes_with_nul(name.as_bytes()).unwrap())
    }
}

impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(Some(self.handle), None);
        }
    }
}
