use crate::vulkan::debug_utils;
use crate::vulkan::instance::Instance;
use erupt::{vk, DeviceLoader, ExtendableFromConst, ExtendableFromMut, InstanceLoader};
use gpu_alloc::{GpuAllocator, MemoryBlock};
use gpu_alloc_erupt::EruptMemoryDevice;
use parking_lot::Mutex;
use std::ffi::CStr;
use std::ops::Deref;

pub struct Device {
    handle: DeviceLoader,
    instance: Instance,
    allocator: Mutex<GpuAllocator<vk::DeviceMemory>>,
    physical_device: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    enabled_features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    supported_extensions: Vec<String>,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    queue_family_indices: QueueFamilyIndices,
    graphics_queue: vk::Queue,
}

impl Device {
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn graphics_family_index(&self) -> u32 {
        self.queue_family_indices.graphics
    }

    pub fn new(enabled_extensions: &[*const i8], requested_queue_types: vk::QueueFlags) -> Self {
        let instance = Instance::new();
        let physical_device = pick_physical_device(instance.handle());

        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device, None) };

        let supported_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device, None, None)
                .unwrap()
                .iter()
                .map(|e| String::from(CStr::from_ptr(e.extension_name.as_ptr()).to_str().unwrap()))
                .collect::<Vec<String>>()
        };

        let mut queue_create_infos = vec![];
        let default_queue_priority = [0.0];
        let mut queue_family_indices = QueueFamilyIndices::default();

        if requested_queue_types.contains(vk::QueueFlags::GRAPHICS) {
            queue_family_indices.graphics =
                get_queue_family_index(vk::QueueFlags::GRAPHICS, &queue_family_properties);
            let queue_info = vk::DeviceQueueCreateInfoBuilder::new()
                .queue_family_index(queue_family_indices.graphics)
                .queue_priorities(&default_queue_priority);
            queue_create_infos.push(queue_info);
        } else {
            unimplemented!()
        }

        let enabled_features = vk::PhysicalDeviceFeatures::default();

        let mut device_extensions = enabled_extensions.to_vec();
        unsafe {
            if supported_extensions.contains(&String::from(
                CStr::from_ptr(vk::EXT_DEBUG_MARKER_EXTENSION_NAME)
                    .to_str()
                    .unwrap(),
            )) {
                device_extensions.push(vk::EXT_DEBUG_MARKER_EXTENSION_NAME);
            }
        }

        let mut device_layers = vec![];
        if cfg!(debug_assertions) {
            device_layers.push(debug_utils::VALIDATION_LAYER);
        }

        let buffer_device_address_features =
            vk::PhysicalDeviceBufferDeviceAddressFeaturesBuilder::new().buffer_device_address(true);
        let indexing_features = vk::PhysicalDeviceDescriptorIndexingFeaturesBuilder::new()
            .runtime_descriptor_array(true);
        let reset_query_features =
            vk::PhysicalDeviceHostQueryResetFeaturesBuilder::new().host_query_reset(true);
        let acceleration_structure_features =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHRBuilder::new()
                .acceleration_structure(true);
        let ray_tracing_features = vk::PhysicalDeviceRayTracingPipelineFeaturesKHRBuilder::new()
            .ray_tracing_pipeline(true);
        let vulkan_memory_model_features =
            vk::PhysicalDeviceVulkanMemoryModelFeaturesBuilder::new().vulkan_memory_model(true);

        let device_create_info = vk::DeviceCreateInfoBuilder::new()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&enabled_features)
            .enabled_extension_names(&device_extensions)
            .enabled_layer_names(&device_layers)
            .extend_from(&buffer_device_address_features)
            .extend_from(&indexing_features)
            .extend_from(&reset_query_features)
            .extend_from(&acceleration_structure_features)
            .extend_from(&ray_tracing_features)
            .extend_from(&vulkan_memory_model_features);

        let device = unsafe {
            DeviceLoader::new(
                instance.handle(),
                physical_device,
                &device_create_info,
                None,
            )
            .unwrap()
        };

        let allocator = Mutex::new(GpuAllocator::new(
            gpu_alloc::Config::i_am_prototyping(),
            unsafe {
                gpu_alloc_erupt::device_properties(instance.handle(), physical_device).unwrap()
            },
        ));

        let graphics_queue = unsafe { device.get_device_queue(queue_family_indices.graphics, 0) };

        Device {
            handle: device,
            allocator,
            instance,
            physical_device,
            properties,
            features,
            enabled_features,
            memory_properties,
            supported_extensions,
            queue_family_properties,
            queue_family_indices,
            graphics_queue,
        }
    }

    pub fn get_supported_depth_format(&self) -> vk::Format {
        let depth_formats = [
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D32_SFLOAT,
            vk::Format::D24_UNORM_S8_UINT,
            vk::Format::D16_UNORM_S8_UINT,
            vk::Format::D16_UNORM,
        ];

        *depth_formats
            .iter()
            .find(|format| {
                let format_props = unsafe {
                    self.instance
                        .get_physical_device_format_properties(self.physical_device, **format)
                };
                format_props
                    .optimal_tiling_features
                    .contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)
            })
            .unwrap()
    }

    pub fn gpu_alloc_memory(
        &self,
        mem_reqs: vk::MemoryRequirements,
    ) -> gpu_alloc::MemoryBlock<vk::DeviceMemory> {
        unsafe {
            self.allocator
                .lock()
                .alloc(
                    EruptMemoryDevice::wrap(&self.handle),
                    gpu_alloc::Request {
                        size: mem_reqs.size,
                        align_mask: mem_reqs.alignment - 1,
                        usage: gpu_alloc::UsageFlags::empty(),
                        memory_types: mem_reqs.memory_type_bits,
                    },
                )
                .unwrap()
        }
    }

    pub fn gpu_dealloc_memory(&self, mem_block: MemoryBlock<vk::DeviceMemory>) {
        unsafe {
            self.allocator
                .lock()
                .dealloc(EruptMemoryDevice::wrap(&self.handle), mem_block);
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        log::debug!("Dropping device");
        unsafe {
            self.handle.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl Deref for Device {
    type Target = DeviceLoader;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

fn pick_physical_device(instance: &InstanceLoader) -> vk::PhysicalDevice {
    let physical_devices = unsafe { instance.enumerate_physical_devices(None).unwrap() };
    physical_devices[0]
}

fn get_queue_family_index(
    queue_flag: vk::QueueFlags,
    queue_family_properties: &[vk::QueueFamilyProperties],
) -> u32 {
    queue_family_properties
        .iter()
        .enumerate()
        .find(|(_, queue)| queue.queue_flags.contains(queue_flag))
        .unwrap()
        .0 as _
}

#[derive(Default)]
pub struct QueueFamilyIndices {
    pub graphics: u32,
    pub compute: u32,
    pub transfer: u32,
}
