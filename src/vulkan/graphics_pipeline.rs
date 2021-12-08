use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::descriptor_set_manager::DescriptorSetManager;
use crate::vulkan::device::Device;
use crate::vulkan::pipeline_layout::{PipelineLayout, PushConstantRanges};
use crate::vulkan::render_pass::RenderPass;
use crate::vulkan::shader_module::ShaderModule;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::uniform_buffer::UniformBuffer;
use crate::vulkan::vertex::{EguiVertex, ModelVertex, Vertex};
use bytemuck::{Pod, Zeroable};
use erupt::vk;
use glam::Vec2;
use std::mem::size_of;
use std::rc::Rc;

struct PushConstant {
    screen_size: Vec2,
}

pub struct GraphicsPipeline {
    handle: vk::Pipeline,
    device: Rc<Device>,
    descriptor_set_manager: Option<DescriptorSetManager>,
    pipeline_layout: PipelineLayout,
}

impl GraphicsPipeline {
    pub fn handle(&self) -> vk::Pipeline {
        self.handle
    }

    pub fn pipeline_layout(&self) -> &PipelineLayout {
        &self.pipeline_layout
    }

    pub fn descriptor_set_manager(&self) -> &DescriptorSetManager {
        self.descriptor_set_manager.as_ref().unwrap()
    }

    pub fn uninitialized(device: Rc<Device>) -> Self {
        let pipeline_layout = PipelineLayout::uninitialized(device.clone());
        GraphicsPipeline {
            handle: Default::default(),
            device,
            descriptor_set_manager: None,
            pipeline_layout,
        }
    }

    pub fn new(
        device: Rc<Device>,
        swapchain: &Swapchain,
        render_pass: &RenderPass,
        uniform_buffers: &[UniformBuffer],
        is_wireframe: bool,
    ) -> Self {
        let binding_descriptions = ModelVertex::binding_descriptions();
        let attribute_descriptions = ModelVertex::attribute_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfoBuilder::new()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        //
        let viewports = [vk::ViewportBuilder::new()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent().width as _)
            .height(swapchain.extent().height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];

        let scissors = [vk::Rect2DBuilder::new()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent())];

        let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(if is_wireframe {
                vk::PolygonMode::LINE
            } else {
                vk::PolygonMode::FILL
            })
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlagBits::_1);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfoBuilder::new()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentStateBuilder::new()
            .color_write_mask(vk::ColorComponentFlags::all())
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .blend_enable(true)];

        let color_blending = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .attachments(&color_blend_attachments);

        let descriptor_bindings = [
            DescriptorBinding::new(
                0,
                1,
                vk::DescriptorType::UNIFORM_BUFFER,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            ),
            // DescriptorBinding::new(
            //     1,
            //     1,
            //     vk::DescriptorType::STORAGE_BUFFER,
            //     vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            // ),
            // DescriptorBinding::new(
            //     2,
            //     1,
            //     vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            //     vk::ShaderStageFlags::FRAGMENT,
            // ),
        ];

        let descriptor_set_manager = DescriptorSetManager::new(
            device.clone(),
            &descriptor_bindings,
            uniform_buffers.len() as u32,
        );

        swapchain.images().iter().enumerate().for_each(|(i, _)| {
            let uniform_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
                .buffer(uniform_buffers[i].buffer().handle())
                .range(vk::WHOLE_SIZE)];

            // let material_buffer_info = [vk::DescriptorBufferInfoBuilder::new()
            //     .buffer(scene.material_buffer().handle())
            //     .range(vk::WHOLE_SIZE)];
            //
            // let image_info = scene
            //     .texture_sampler_handles()
            //     .iter()
            //     .map(|_| {
            //         vk::DescriptorImageInfoBuilder::new()
            //             .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            //             .image_view(vk::ImageView::null())
            //             .sampler(vk::Sampler::null())
            //     })
            //     .collect::<Vec<_>>();

            let descriptor_writes = [
                descriptor_set_manager.bind_buffer(i as _, 0, &uniform_buffer_info),
                // descriptor_set_manager.bind_buffer(i as _, 1, &material_buffer_info),
                // descriptor_set_manager.bind_image(i as _, 2, &image_info),
            ];

            descriptor_set_manager.update_descriptors(&descriptor_writes);
        });

        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            &[descriptor_set_manager.descriptor_set_layout()],
            &[],
        );

        let shader_module = ShaderModule::new(device.clone(), "raster");
        let shader_stages = [
            shader_module.create_shader_stage(vk::ShaderStageFlagBits::VERTEX, "main_vs\0"),
            shader_module.create_shader_stage(vk::ShaderStageFlagBits::FRAGMENT, "main_fs\0"),
        ];

        let pipeline_info = [vk::GraphicsPipelineCreateInfoBuilder::new()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout.handle())
            .render_pass(render_pass.handle())
            .subpass(0)];

        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(None, &pipeline_info, None)
                .unwrap()[0]
        };

        GraphicsPipeline {
            handle: graphics_pipeline,
            device,
            descriptor_set_manager: Some(descriptor_set_manager),
            pipeline_layout,
        }
    }

    pub fn new_ui(device: Rc<Device>, swapchain: &Swapchain, render_pass: &RenderPass) -> Self {
        let binding_descriptions = EguiVertex::binding_descriptions();
        let attribute_descriptions = EguiVertex::attribute_descriptions();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfoBuilder::new()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        //
        let viewports = [vk::ViewportBuilder::new()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent().width as _)
            .height(swapchain.extent().height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];

        let scissors = [vk::Rect2DBuilder::new()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent())];

        let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlagBits::_1);

        let stencil_op = vk::StencilOpStateBuilder::new()
            .fail_op(vk::StencilOp::KEEP)
            .pass_op(vk::StencilOp::KEEP)
            .compare_op(vk::CompareOp::ALWAYS)
            .build();
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfoBuilder::new()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::ALWAYS)
            .depth_bounds_test_enable(false)
            .front(stencil_op)
            .back(stencil_op);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentStateBuilder::new()
            .color_write_mask(vk::ColorComponentFlags::all())
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .blend_enable(true)];

        let color_blending = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .attachments(&color_blend_attachments);

        let push_constant_ranges = PushConstantRanges::new(
            vk::ShaderStageFlags::VERTEX,
            0,
            size_of::<PushConstant>() as _,
        );

        let descriptor_bindings = [DescriptorBinding::new(
            0,
            1,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            vk::ShaderStageFlags::FRAGMENT,
        )];

        let descriptor_set_manager =
            DescriptorSetManager::new(device.clone(), &descriptor_bindings, 3);

        // swapchain.images().iter().enumerate().for_each(|(i, _)| {
        //     let image_info = [vk::DescriptorImageInfoBuilder::new()
        //         .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        //         .image_view(font_image.image_view().handle())
        //         .sampler(font_image.sampler().handle())];
        //
        //     let descriptor_writes = [descriptor_set_manager.bind_image(i as _, 0, &image_info)];
        //
        //     descriptor_set_manager.update_descriptors(&descriptor_writes);
        // });

        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            &[descriptor_set_manager.descriptor_set_layout()],
            &[push_constant_ranges],
        );

        let shader_module = ShaderModule::new(device.clone(), "ui");
        let shader_stages = [
            shader_module.create_shader_stage(
                vk::ShaderStageFlagBits::VERTEX,
                "main_vs\0",
                // CStr::from_bytes_with_nul(b"main_vs\0").unwrap(),
            ),
            shader_module.create_shader_stage(
                vk::ShaderStageFlagBits::FRAGMENT,
                "main_fs\0",
                // CStr::from_bytes_with_nul(b"main_fs\0").unwrap(),
            ),
        ];

        let pipeline_info = [vk::GraphicsPipelineCreateInfoBuilder::new()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout.handle())
            .render_pass(render_pass.handle())
            .subpass(0)];

        let graphics_pipeline = unsafe {
            device
                .create_graphics_pipelines(None, &pipeline_info, None)
                .unwrap()[0]
        };

        GraphicsPipeline {
            handle: graphics_pipeline,
            device,
            descriptor_set_manager: Some(descriptor_set_manager),
            pipeline_layout,
        }
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(Some(self.handle), None);
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct PushConstants {
    pub screen_size: Vec2,
}

unsafe impl Zeroable for PushConstants {}

unsafe impl Pod for PushConstants {}
