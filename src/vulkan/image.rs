use crate::vulkan::buffer::Buffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::depth_buffer::DepthBuffer;
use crate::vulkan::device::Device;
use crate::vulkan::device_memory::DeviceMemory;
use erupt::vk;
use std::ops::Deref;
use std::rc::Rc;

pub struct Image {
    handle: vk::Image,
    device_memory: Option<DeviceMemory>,
    device: Rc<Device>,
    extent: vk::Extent2D,
    format: vk::Format,
    image_layout: vk::ImageLayout,
}

impl Image {
    pub fn handle(&self) -> vk::Image {
        self.handle
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn uninitialized(device: Rc<Device>) -> Self {
        Image {
            handle: Default::default(),
            device_memory: None,
            device,
            extent: Default::default(),
            format: Default::default(),
            image_layout: Default::default(),
        }
    }

    pub fn new(
        device: Rc<Device>,
        extent: vk::Extent2D,
        format: vk::Format,
        tiling: Option<vk::ImageTiling>,
        usage: Option<vk::ImageUsageFlags>,
    ) -> Self {
        let image_layout = vk::ImageLayout::UNDEFINED;
        let create_info = vk::ImageCreateInfoBuilder::new()
            .image_type(vk::ImageType::_2D)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(tiling.unwrap_or(vk::ImageTiling::OPTIMAL))
            .initial_layout(image_layout)
            .usage(
                usage.unwrap_or(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED),
            )
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlagBits::_1);

        let image = unsafe { device.create_image(&create_info, None).unwrap() };

        Image {
            handle: image,
            device_memory: None,
            device,
            extent,
            format,
            image_layout,
        }
    }

    pub fn allocate_memory(&mut self) {
        let mem_reqs = self.get_memory_requirements();

        let device_memory = DeviceMemory::new(
            self.device.clone(),
            mem_reqs,
            gpu_alloc::UsageFlags::empty(),
        );
        device_memory.bind_to_image(self.handle);

        self.device_memory = Some(device_memory);
    }

    pub fn transition_image_layout(
        &mut self,
        command_buffers: &CommandPool,
        new_layout: vk::ImageLayout,
    ) {
        command_buffers.single_time_submit(|command_buffer| {
            let mut aspect_mask;
            if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
                aspect_mask = vk::ImageAspectFlags::DEPTH;
                if DepthBuffer::has_stencil_component(self.format) {
                    aspect_mask |= vk::ImageAspectFlags::STENCIL;
                }
            } else {
                aspect_mask = vk::ImageAspectFlags::COLOR;
            };
            let mut barrier = vk::ImageMemoryBarrierBuilder::new()
                .old_layout(self.image_layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.handle)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let source_stage;
            let destination_stage;
            match self.image_layout {
                vk::ImageLayout::UNDEFINED
                    if new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL =>
                {
                    barrier.src_access_mask = vk::AccessFlags::empty();
                    barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
                    source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
                    destination_stage = vk::PipelineStageFlags::TRANSFER;
                }
                vk::ImageLayout::UNDEFINED
                    if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL =>
                {
                    barrier.src_access_mask = vk::AccessFlags::empty();
                    barrier.dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                        | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
                    source_stage = vk::PipelineStageFlags::TRANSFER;
                    destination_stage = vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
                }
                vk::ImageLayout::TRANSFER_DST_OPTIMAL
                    if new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL =>
                {
                    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
                    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
                    source_stage = vk::PipelineStageFlags::TRANSFER;
                    destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
                }
                _ => unimplemented!(),
            }

            unsafe {
                self.device.cmd_pipeline_barrier(
                    command_buffer,
                    source_stage,
                    destination_stage,
                    None,
                    &[],
                    &[],
                    &[barrier],
                )
            }
        });

        self.image_layout = new_layout;
    }

    pub fn copy_from(&self, command_pool: &CommandPool, buffer: &Buffer) {
        command_pool.single_time_submit(|command_buffer| {
            let region = vk::BufferImageCopyBuilder::new()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: 0,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width: self.extent.width,
                    height: self.extent.height,
                    depth: 1,
                });

            unsafe {
                self.device.cmd_copy_buffer_to_image(
                    command_buffer,
                    buffer.handle(),
                    self.handle,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[region],
                );
            }
        });
    }

    fn get_memory_requirements(&self) -> vk::MemoryRequirements {
        unsafe { self.device.get_image_memory_requirements(self.handle) }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image(Some(self.handle), None);
        }
    }
}

impl Deref for Image {
    type Target = vk::Image;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
