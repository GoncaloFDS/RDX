use crate::renderers::Renderer;
use crate::user_interface::UserInterface;
use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::descriptor_set_manager::DescriptorSetManager;
use crate::vulkan::device::Device;
use crate::vulkan::pipeline_layout::PipelineLayout;
use crate::vulkan::pipelines::graphics_pipeline::GraphicsPipeline;
use crate::vulkan::push_constants::PushConstantRanges;
use crate::vulkan::shader_module::{Shader, ShaderModule};
use crate::vulkan::texture::Texture;
use crate::vulkan::texture_image::TextureImage;
use crate::vulkan::vertex::{EguiVertex, Vertex};
use bytemuck::{Pod, Zeroable};
use egui::{ImageData, TextureId};
use erupt::{vk, ExtendableFrom};
use glam::{vec2, vec4, Vec2};
use std::collections::HashMap;
use std::mem::size_of;
use std::slice;

const VERTICES_PER_QUAD: u64 = 4;
const VERTEX_BUFFER_SIZE: u64 = 1024 * 1024 * VERTICES_PER_QUAD;
const INDEX_BUFFER_SIZE: u64 = 1024 * 1024 * 2;

#[derive(Copy, Clone, Default)]
struct EguiPushConstant {
    screen_size: Vec2,
}

unsafe impl Zeroable for EguiPushConstant {}
unsafe impl Pod for EguiPushConstant {}

pub struct DrawIndexed {
    pub vertex_offset: u64,
    pub index_offset: u64,
    pub index_count: u32,
    pub id: usize,
}

pub struct EguiRenderer {
    pipeline: GraphicsPipeline,
    pipeline_layout: PipelineLayout,
    descriptor_set_manager: DescriptorSetManager,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    draw_indexed: Vec<DrawIndexed>,
    textures: HashMap<TextureId, TextureImage>,
}

impl EguiRenderer {
    pub fn new(device: &mut Device, _ui: &UserInterface) -> Self {
        let binding_descriptions = EguiVertex::binding_descriptions();
        let attribute_descriptions = EguiVertex::attribute_descriptions();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfoBuilder::new()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let shader_module = ShaderModule::new(device, Shader::UI);

        let shader_stages = [
            shader_module.shader_stage(vk::ShaderStageFlagBits::VERTEX, "main_vs\0"),
            shader_module.shader_stage(vk::ShaderStageFlagBits::FRAGMENT, "main_fs\0"),
        ];

        let surface_format = device.surface_format().format;
        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfoBuilder::new()
            .color_attachment_formats(slice::from_ref(&surface_format));

        let dynamic_pipeline_state = vk::PipelineDynamicStateCreateInfoBuilder::new()
            .dynamic_states(&[vk::DynamicState::SCISSOR, vk::DynamicState::VIEWPORT]);

        let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
            .scissor_count(1)
            .viewport_count(1);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfoBuilder::new()
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
            size_of::<EguiPushConstant>() as _,
        );

        let descriptor_bindings = [DescriptorBinding::new(
            0,
            1,
            vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            vk::ShaderStageFlags::FRAGMENT,
        )];

        let descriptor_set_manager = DescriptorSetManager::new(device, &descriptor_bindings, 3);

        let pipeline_layout = PipelineLayout::new(
            device,
            &[descriptor_set_manager.descriptor_set_layout()],
            &[push_constant_ranges],
        );

        let pipeline_info = &[vk::GraphicsPipelineCreateInfoBuilder::new()
            .vertex_input_state(&vertex_input_state)
            .color_blend_state(&color_blending)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil)
            .stages(&shader_stages)
            .layout(pipeline_layout.handle())
            .rasterization_state(&rasterization_state)
            .dynamic_state(&dynamic_pipeline_state)
            .viewport_state(&viewport_state)
            .input_assembly_state(&input_assembly_state)
            .extend_from(&mut pipeline_rendering_info)];

        let pipeline = GraphicsPipeline::new(device, pipeline_info, vk::PipelineCache::null());

        shader_module.destroy(device);

        // buffer allocation

        let vertex_buffer = Buffer::empty(
            device,
            VERTEX_BUFFER_SIZE * size_of::<EguiVertex>() as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            gpu_alloc::UsageFlags::HOST_ACCESS | gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        let index_buffer = Buffer::empty(
            device,
            INDEX_BUFFER_SIZE * size_of::<u32>() as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            gpu_alloc::UsageFlags::HOST_ACCESS | gpu_alloc::UsageFlags::DEVICE_ADDRESS,
        );

        EguiRenderer {
            pipeline,
            pipeline_layout,
            descriptor_set_manager,
            vertex_buffer,
            index_buffer,
            draw_indexed: vec![],
            textures: Default::default(),
        }
    }

    fn update_buffers(&mut self, device: &Device, ui: &mut UserInterface) {
        let mut vertex_start = 0;
        let mut index_start = 0;
        self.draw_indexed.clear();
        for egui::ClippedMesh(_rect, mesh) in ui.clipped_meshes() {
            if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                continue;
            }

            let vertices = mesh
                .vertices
                .iter()
                .map(|vertex| {
                    EguiVertex::new(
                        vec2(vertex.pos.x, vertex.pos.y),
                        vec2(vertex.uv.x, vertex.uv.y),
                        vec4(
                            vertex.color.r() as f32 / 255.0,
                            vertex.color.g() as f32 / 255.0,
                            vertex.color.b() as f32 / 255.0,
                            vertex.color.a() as f32 / 255.0,
                        ),
                    )
                })
                .collect::<Vec<_>>();

            self.vertex_buffer
                .write_data(device, &vertices, vertex_start);
            self.index_buffer
                .write_data(device, &mesh.indices, index_start);

            self.draw_indexed.push(DrawIndexed {
                vertex_offset: vertex_start * size_of::<EguiVertex>() as u64,
                index_offset: index_start * size_of::<u32>() as u64,
                index_count: mesh.indices.len() as u32,
                id: 0,
            });

            vertex_start += mesh.vertices.len() as u64;
            index_start += mesh.indices.len() as u64;
        }
    }

    fn update_textures(&mut self, device: &mut Device, ui: &UserInterface) {
        let textures_delta = ui.textures_delta();
        for texture_id in &textures_delta.free {
            // free texture
            todo!()
        }

        for (texture_id, delta) in &textures_delta.set {
            if self.textures.contains_key(texture_id) {
                return;
            }
            let image_data = match &delta.image {
                ImageData::Color(image) => image
                    .pixels
                    .iter()
                    .flat_map(|c| c.to_array())
                    .collect::<Vec<_>>(),

                ImageData::Alpha(image) => image
                    .pixels
                    .iter()
                    .flat_map(|&c| vec![c, c, c, c])
                    .collect::<Vec<_>>(),
            };
            let texture = Texture::new(
                delta.image.width() as u32,
                delta.image.height() as u32,
                image_data,
            );
            let texture_image = TextureImage::new(device, &texture);

            (0..3).enumerate().for_each(|(i, _)| {
                let image_info = [vk::DescriptorImageInfoBuilder::new()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(texture_image.image_view().handle())
                    .sampler(texture_image.sampler().handle())];
                let descriptor_writes =
                    [self
                        .descriptor_set_manager
                        .bind_image(i as _, 0, &image_info)];

                self.descriptor_set_manager
                    .update_descriptors(device, &descriptor_writes);
            });

            self.textures.insert(*texture_id, texture_image);
        }
    }
}

impl Renderer for EguiRenderer {
    fn fill_command_buffer(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
        current_image: usize,
    ) {
        command_buffer.bind_pipeline(
            device,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.handle(),
        );

        command_buffer.bind_descriptor_sets(
            device,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout.handle(),
            &[self.descriptor_set_manager.descriptor_set(current_image)],
        );

        let extent = device.swapchain().extent();
        let push_constants = EguiPushConstant {
            screen_size: vec2(extent.width as f32, extent.height as f32),
        };

        command_buffer.push_constants(
            device,
            self.pipeline_layout.handle(),
            vk::ShaderStageFlags::VERTEX,
            0,
            &push_constants,
        );

        for draw_indexed in &self.draw_indexed {
            command_buffer.bind_vertex_buffer(
                device,
                &[&self.vertex_buffer],
                &[draw_indexed.vertex_offset],
            );
            command_buffer.bind_index_buffer(device, &self.index_buffer, draw_indexed.index_offset);
            command_buffer.draw_indexed(device, draw_indexed.index_count);
        }
    }

    fn update(&mut self, device: &mut Device, ui: &mut UserInterface) {
        self.update_buffers(device, ui);
        self.update_textures(device, ui);
    }

    fn destroy(&mut self, device: &mut Device) {
        self.textures
            .iter_mut()
            .for_each(|(_, texture)| texture.destroy(device));
        self.descriptor_set_manager.destroy(device);
        self.vertex_buffer.destroy(device);
        self.index_buffer.destroy(device);
        self.pipeline_layout.destroy(device);
        self.pipeline.destroy(device);
    }
}
