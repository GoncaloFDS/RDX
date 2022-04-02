use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::pipeline_layout::PipelineLayout;
use crate::vulkan::pipelines::graphics_pipeline::GraphicsPipeline;
use crate::vulkan::shader_module::{Shader, ShaderModule};
use erupt::{vk, ExtendableFrom};
use std::slice;

pub struct TrianglePass {
    pub pipeline: GraphicsPipeline,
    pub pipeline_layout: PipelineLayout,
}

impl TrianglePass {
    pub fn new(device: &Device, surface_format: vk::SurfaceFormatKHR) -> Self {
        // Pipeline creation
        let shader_module = ShaderModule::new(device, Shader::Raster);

        let shader_stages = [
            shader_module.shader_stage(vk::ShaderStageFlagBits::VERTEX, "main_vs\0"),
            shader_module.shader_stage(vk::ShaderStageFlagBits::FRAGMENT, "main_fs\0"),
        ];

        let pipeline_layout = PipelineLayout::new(device, &[], &[]);

        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfoBuilder::new()
            .color_attachment_formats(slice::from_ref(&surface_format.format));

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let dynamic_pipeline_state = vk::PipelineDynamicStateCreateInfoBuilder::new()
            .dynamic_states(&[vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT]);

        let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
            .scissor_count(1)
            .viewport_count(1);
        let rasterization_state =
            vk::PipelineRasterizationStateCreateInfoBuilder::new().line_width(1.0);
        let multisample_state = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .rasterization_samples(vk::SampleCountFlagBits::_1);

        let color_blend_attachments = vec![vk::PipelineColorBlendAttachmentStateBuilder::new()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(false)];

        let color_blending_info = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfoBuilder::new();
        let pipeline_infos = &[vk::GraphicsPipelineCreateInfoBuilder::new()
            .vertex_input_state(&vertex_input_state)
            .color_blend_state(&color_blending_info)
            .multisample_state(&multisample_state)
            .stages(&shader_stages)
            .layout(pipeline_layout.handle())
            .rasterization_state(&rasterization_state)
            .dynamic_state(&dynamic_pipeline_state)
            .viewport_state(&viewport_state)
            .input_assembly_state(&input_assembly_state)
            .extend_from(&mut pipeline_rendering_info)];

        let pipeline = GraphicsPipeline::new(device, pipeline_infos, vk::PipelineCache::null());

        shader_module.destroy(device);

        TrianglePass {
            pipeline,
            pipeline_layout,
        }
    }

    pub fn draw(&self, device: &Device, command_buffer: CommandBuffer) {
        command_buffer.draw(device, 3, 1, 0, 0);
    }

    pub fn destroy(&self, device: &Device) {
        self.pipeline.destroy(device);
        self.pipeline_layout.destroy(device);
    }
}
