use crate::vulkan::instance::Instance;
use crate::vulkan::surface::Surface;
use erupt::{vk, DeviceLoader, ExtendableFrom};
use erupt_bootstrap::{DeviceBuilder, DeviceMetadata, QueueFamilyCriteria};
use gpu_alloc::GpuAllocator;
use gpu_alloc_erupt::EruptMemoryDevice;

pub struct Device {
    handle: DeviceLoader,
    metadata: DeviceMetadata,
    queue: vk::Queue,
    queue_index: u32,
    allocator: GpuAllocator<vk::DeviceMemory>,
}

impl Device {
    pub fn new(instance: &Instance, surface: Surface) -> Self {
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
            .for_surface(surface.handle())
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

        Device {
            handle: device,
            metadata,
            queue,
            queue_index,
            allocator,
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
