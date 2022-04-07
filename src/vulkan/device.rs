use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::command_pool::CommandPool;
use crate::vulkan::frame::Frame;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::instance::Instance;
use crate::vulkan::semaphore::Semaphore;
use crate::vulkan::swapchain::Swapchain;
use erupt::{vk, DeviceLoader, ExtendableFrom};
use erupt_bootstrap::{AcquiredFrame, DeviceBuilder, DeviceMetadata, QueueFamilyCriteria};
use gpu_alloc::GpuAllocator;
use gpu_alloc_erupt::EruptMemoryDevice;
use winit::window::Window;

pub struct Device {
    handle: DeviceLoader,
    metadata: DeviceMetadata,
    queue: vk::Queue,
    queue_index: u32,
    allocator: GpuAllocator<vk::DeviceMemory>,
    swapchain: Swapchain,
    swapchain_image_views: Vec<ImageView>,
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

        let features = vk::PhysicalDeviceFeatures2Builder::new()
            .extend_from(&mut vk1_2features)
            .extend_from(&mut vk1_3features);

        let device_builder = DeviceBuilder::new()
            .require_version(1, 3)
            .require_extension(vk::KHR_SWAPCHAIN_EXTENSION_NAME)
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

    pub fn destroy(&self) {
        unsafe {
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

    pub fn swapchain_image(&self, current_image: usize) -> vk::Image {
        self.swapchain.images()[current_image]
    }

    pub fn swapchain_image_view(&self, current_image: usize) -> &ImageView {
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
            image_view.destoy(&self.handle)
        }

        let format = self.swapchain.format();
        self.swapchain_image_views = self
            .swapchain
            .images()
            .iter()
            .map(|&swapchain_image| {
                ImageView::new(
                    &self,
                    swapchain_image,
                    format.format,
                    vk::ImageAspectFlags::COLOR,
                )
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
}
