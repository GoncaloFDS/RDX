use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::frame::Frame;
use crate::vulkan::instance::Instance;
use crate::vulkan::semaphore::Semaphore;
use crate::vulkan::swapchain::Swapchain;
use erupt::{vk, DeviceLoader, ExtendableFrom};
use erupt_bootstrap::{AcquiredFrame, DeviceBuilder, DeviceMetadata, QueueFamilyCriteria};
use gpu_alloc::{GpuAllocator, MemoryBlock};
use gpu_alloc_erupt::EruptMemoryDevice;
use winit::window::Window;

pub struct Device {
    handle: DeviceLoader,
    metadata: DeviceMetadata,
    queue: vk::Queue,
    queue_index: u32,
    allocator: GpuAllocator<vk::DeviceMemory>,
    swapchain: Swapchain,
    swapchain_image_views: Vec<vk::ImageView>,
    command_pool: CommandPool,
    frames: Vec<Frame>,
}

impl Device {
    pub fn new(instance: &Instance, window: &Window) -> Self {
        let graphics_present = QueueFamilyCriteria::graphics_present();

        let mut vk1_2features = vk::PhysicalDeviceVulkan12FeaturesBuilder::new()
            .vulkan_memory_model(true)
            .buffer_device_address(true)
            .runtime_descriptor_array(true);

        let mut vk1_3features = vk::PhysicalDeviceVulkan13FeaturesBuilder::new()
            .dynamic_rendering(true)
            .synchronization2(true);

        let mut acceleration_structure_features =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHRBuilder::new()
                .acceleration_structure(true);
        let mut ray_tracing_features =
            vk::PhysicalDeviceRayTracingPipelineFeaturesKHRBuilder::new()
                .ray_tracing_pipeline(true);

        let features = vk::PhysicalDeviceFeatures2Builder::new()
            .extend_from(&mut vk1_2features)
            .extend_from(&mut vk1_3features)
            .extend_from(&mut acceleration_structure_features)
            .extend_from(&mut ray_tracing_features);

        let device_builder = DeviceBuilder::new()
            .require_version(1, 3)
            .require_extension(vk::KHR_SWAPCHAIN_EXTENSION_NAME)
            .require_extension(vk::KHR_ACCELERATION_STRUCTURE_EXTENSION_NAME)
            .require_extension(vk::KHR_RAY_TRACING_PIPELINE_EXTENSION_NAME)
            .require_extension(vk::KHR_BUFFER_DEVICE_ADDRESS_EXTENSION_NAME)
            .require_extension(vk::KHR_DEFERRED_HOST_OPERATIONS_EXTENSION_NAME)
            .queue_family(graphics_present)
            .for_surface(instance.surface().handle())
            .require_features(&features);

        let (device, metadata) = unsafe {
            device_builder
                .build(instance.handle(), instance.metadata())
                .unwrap()
        };

        let (queue, queue_index) = metadata
            .device_queue(instance.handle(), &device, graphics_present, 0)
            .unwrap()
            .unwrap();

        let allocator_properties = unsafe {
            gpu_alloc_erupt::device_properties(instance.handle(), metadata.physical_device())
                .unwrap()
        };
        let allocator =
            GpuAllocator::new(gpu_alloc::Config::i_am_prototyping(), allocator_properties);

        let swapchain = Swapchain::new(
            window,
            instance,
            instance.surface(),
            &device,
            metadata.physical_device(),
        );

        let command_pool = CommandPool::new(&device, queue_index, true);
        let command_buffers = command_pool.allocate(&device, swapchain.frames_in_flight() as u32);

        let frames = command_buffers
            .iter()
            .map(|&command_buffer| {
                let cmd_complete_semaphore = Semaphore::new(&device);
                Frame::new(command_buffer, cmd_complete_semaphore)
            })
            .collect();

        Device {
            handle: device,
            metadata,
            queue,
            queue_index,
            allocator,
            swapchain,
            swapchain_image_views: vec![],
            command_pool,
            frames,
        }
    }

    pub fn destroy(&mut self) {
        for frame in &self.frames {
            frame.destroy(self);
        }

        for swapchain_image_view in &self.swapchain_image_views {
            unsafe {
                self.handle.destroy_image_view(*swapchain_image_view, None);
            }
        }
        self.command_pool.free_command_buffers(
            &self,
            &self
                .frames
                .iter()
                .map(|frame| frame.command_buffer)
                .collect::<Vec<_>>(),
        );
        self.command_pool.destroy(&self);
        unsafe {
            self.swapchain.as_mut().destroy(&self.handle);
            self.handle.destroy_device(None);
        }
    }

    pub fn handle(&self) -> &DeviceLoader {
        &self.handle
    }

    pub fn metadata(&self) -> &DeviceMetadata {
        &self.metadata
    }

    pub fn queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn queue_index(&self) -> u32 {
        self.queue_index
    }

    pub fn swapchain(&self) -> &Swapchain {
        &self.swapchain
    }

    pub fn command_pool(&self) -> &CommandPool {
        &self.command_pool
    }

    pub fn swapchain_image(&self, current_image: usize) -> vk::Image {
        self.swapchain.images()[current_image]
    }

    pub fn swapchain_image_view(&self, current_image: usize) -> &vk::ImageView {
        &self.swapchain_image_views[current_image]
    }

    pub fn command_buffer(&self, current_image: usize) -> &CommandBuffer {
        &self.frames[current_image].command_buffer
    }

    pub fn semaphore(&self, current_image: usize) -> &Semaphore {
        &self.frames[current_image].semaphore
    }

    pub fn surface_format(&self) -> vk::SurfaceFormatKHR {
        self.swapchain.surface_format()
    }

    pub fn resize_swapchain(&mut self, size: vk::Extent2D) {
        self.swapchain.resize(size);
    }

    pub fn acquire_swapchain_frame(&mut self, instance: &Instance, timeout: u64) -> AcquiredFrame {
        self.swapchain.acquire(instance, &self.handle, timeout)
    }

    pub fn recreate_swapchain(&mut self) {
        for image_view in &self.swapchain_image_views {
            unsafe {
                self.handle.destroy_image_view(*image_view, None);
            }
        }

        let format = self.swapchain.format();
        self.swapchain_image_views = self
            .swapchain
            .images()
            .iter()
            .map(|&swapchain_image| {
                let create_info = vk::ImageViewCreateInfoBuilder::new()
                    .image(swapchain_image)
                    .view_type(vk::ImageViewType::_2D)
                    .format(format.format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe { self.handle.create_image_view(&create_info, None).unwrap() }
            })
            .collect()
    }

    pub fn queue_present(
        &mut self,
        queue: vk::Queue,
        render_complete: vk::Semaphore,
        image_index: usize,
    ) {
        self.swapchain
            .queue_present(&self.handle, queue, render_complete, image_index);
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.handle.device_wait_idle().unwrap();
        }
    }

    pub fn submit(
        &self,
        wait_info: &[vk::SemaphoreSubmitInfoBuilder],
        signal_info: &[vk::SemaphoreSubmitInfoBuilder],
        command_buffer_info: &[vk::CommandBufferSubmitInfoBuilder],
        fence: vk::Fence,
    ) {
        let submit_info = vk::SubmitInfo2Builder::new()
            .wait_semaphore_infos(wait_info)
            .signal_semaphore_infos(signal_info)
            .command_buffer_infos(command_buffer_info);
        unsafe {
            self.handle
                .queue_submit2(self.queue, &[submit_info], fence)
                .unwrap();
        }
    }

    pub fn allocate_memory(
        &mut self,
        memory_requirements: vk::MemoryRequirements,
        allocation_flags: gpu_alloc::UsageFlags,
    ) -> gpu_alloc::MemoryBlock<vk::DeviceMemory> {
        unsafe {
            self.allocator
                .alloc(
                    EruptMemoryDevice::wrap(&self.handle),
                    gpu_alloc::Request {
                        size: memory_requirements.size,
                        align_mask: memory_requirements.alignment - 1,
                        usage: allocation_flags,
                        memory_types: memory_requirements.memory_type_bits,
                    },
                )
                .unwrap()
        }
    }

    pub fn dealloc_memory(&mut self, memory_block: MemoryBlock<vk::DeviceMemory>) {
        unsafe {
            self.allocator
                .dealloc(EruptMemoryDevice::wrap(&self.handle), memory_block);
        }
    }
}
