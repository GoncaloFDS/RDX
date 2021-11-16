use crate::debug::VALIDATION_LAYER;
use erupt::vk1_0::Format;
use erupt::{vk, DeviceLoader, EntryLoader, ExtendableFromConst, InstanceLoader};
use gpu_alloc::{GpuAllocator, MemoryBlock};
use gpu_alloc_erupt::EruptMemoryDevice;
use parking_lot::Mutex;
use std::ffi::{CStr, CString};
use std::ops::Range;
use std::os::raw::c_char;

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
pub struct QueueFamilyIndex {
    pub graphics: u32,
    pub compute: u32,
    pub transfer: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Extent {
    D1 { width: u32 },
    D2 { width: u32, height: u32 },
    D3 { width: u32, height: u32, depth: u32 },
}

pub enum ImageMemory {
    DeviceImage {
        memory_block: MemoryBlock<vk::DeviceMemory>,
    },
    SwapchainImage,
}

pub struct Image {
    handle: vk::Image,
    info: ImageInfo,
    memory: ImageMemory,
}

pub struct SubresourceRange {
    pub aspect_flags: vk::ImageAspectFlags,
    pub mip_levels: Range<u32>,
    pub array_layers: Range<u32>,
}

impl SubresourceRange {
    pub fn new(aspect_flags: vk::ImageAspectFlags, levels: Range<u32>, layers: Range<u32>) -> Self {
        SubresourceRange {
            aspect_flags,
            mip_levels: levels,
            array_layers: layers,
        }
    }

    pub fn whole(info: &ImageInfo) -> Self {
        SubresourceRange {
            aspect_flags: vk::ImageAspectFlags::COLOR,
            mip_levels: 0..info.mip_levels,
            array_layers: 0..info.array_layers,
        }
    }
}

pub struct ImageViewInfo {
    pub image: Image,
    pub view_type: vk::ImageViewType,
    pub subresource_range: SubresourceRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageInfo {
    pub extent: Extent,
    pub format: vk::Format,
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: vk::SampleCountFlagBits,
    pub usage: vk::ImageUsageFlags,
    pub tiling: vk::ImageTiling,
}

impl From<Extent> for vk::ImageType {
    fn from(extent: Extent) -> Self {
        match extent {
            Extent::D1 { .. } => vk::ImageType::_1D,
            Extent::D2 { .. } => vk::ImageType::_2D,
            Extent::D3 { .. } => vk::ImageType::_3D,
        }
    }
}

impl From<Extent> for vk::Extent3D {
    fn from(extent: Extent) -> Self {
        match extent {
            Extent::D1 { width } => vk::Extent3D {
                width,
                height: 0,
                depth: 1,
            },
            Extent::D2 { width, height } => vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            Extent::D3 {
                width,
                height,
                depth,
            } => vk::Extent3D {
                width,
                height,
                depth,
            },
        }
    }
}

pub struct Device {
    handle: DeviceLoader,
    allocator: Mutex<GpuAllocator<vk::DeviceMemory>>,
    instance: InstanceLoader,
    _entry: EntryLoader,
    physical_device: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    enabled_features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    supported_extensions: Vec<String>,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    queue_family_indices: QueueFamilyIndex,
    command_pool: vk::CommandPool,
}

impl Device {
    pub fn new(enabled_extensions: &[*const i8], requested_queue_types: vk::QueueFlags) -> Self {
        let entry = EntryLoader::new().unwrap();
        let instance = create_instance(&entry);
        let physical_device = pick_physical_device(&instance);

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
        let mut queue_family_indices = QueueFamilyIndex::default();

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
            device_layers.push(VALIDATION_LAYER);
        }

        // Hardcoded here because I don't know how to make this configurable
        let mut buffer_device_address_features =
            vk::PhysicalDeviceBufferDeviceAddressFeaturesBuilder::new().buffer_device_address(true);
        let mut indexing_features = vk::PhysicalDeviceDescriptorIndexingFeaturesBuilder::new()
            .runtime_descriptor_array(true);
        let mut reset_query_features =
            vk::PhysicalDeviceHostQueryResetFeaturesBuilder::new().host_query_reset(true);
        let mut acceleration_structure_features =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHRBuilder::new()
                .acceleration_structure(true);
        let mut ray_tracing_features =
            vk::PhysicalDeviceRayTracingPipelineFeaturesKHRBuilder::new()
                .ray_tracing_pipeline(true);

        let device_create_info = vk::DeviceCreateInfoBuilder::new()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&enabled_features)
            .enabled_extension_names(&device_extensions)
            .enabled_layer_names(&device_layers)
            // Hardcoded here because I don't know how to make this configurable
            .extend_from(&mut buffer_device_address_features)
            .extend_from(&mut indexing_features)
            .extend_from(&mut reset_query_features)
            .extend_from(&mut acceleration_structure_features)
            .extend_from(&mut ray_tracing_features);

        let device = unsafe {
            DeviceLoader::new(&instance, physical_device, &device_create_info, None).unwrap()
        };

        let command_pool = unsafe {
            device
                .create_command_pool(
                    &vk::CommandPoolCreateInfoBuilder::new()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(queue_family_indices.graphics),
                    None,
                )
                .unwrap()
        };

        let allocator = Mutex::new(GpuAllocator::new(
            gpu_alloc::Config::i_am_prototyping(),
            unsafe { gpu_alloc_erupt::device_properties(&instance, physical_device).unwrap() },
        ));

        Device {
            handle: device,
            allocator,
            instance,
            _entry: entry,
            physical_device,
            properties,
            features,
            enabled_features,
            memory_properties,
            supported_extensions,
            queue_family_properties,
            queue_family_indices,
            command_pool,
        }
    }

    pub fn get_instance(&self) -> &InstanceLoader {
        &self.instance
    }

    pub fn get_physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn get_device_queue(&self) -> vk::Queue {
        unsafe {
            self.handle
                .get_device_queue(self.queue_family_indices.graphics, 0)
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

    pub fn create_swapchain_khr(
        &self,
        swapchain_create_info: vk::SwapchainCreateInfoKHRBuilder,
    ) -> vk::SwapchainKHR {
        unsafe {
            self.handle
                .create_swapchain_khr(&swapchain_create_info, None)
                .unwrap()
        }
    }

    pub fn destroy_image_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.handle.destroy_image_view(Some(image_view), None);
        }
    }

    pub fn destroy_swapchain_khr(&self, swapchain_khr: vk::SwapchainKHR) {
        unsafe {
            self.handle.destroy_swapchain_khr(Some(swapchain_khr), None);
        }
    }

    pub fn get_swapchain_images_khr(&self, swapchain_khr: vk::SwapchainKHR) -> Vec<vk::Image> {
        unsafe {
            self.handle
                .get_swapchain_images_khr(swapchain_khr, None)
                .unwrap()
        }
    }

    pub fn create_image_view(&self, create_info: vk::ImageViewCreateInfo) -> vk::ImageView {
        unsafe { self.handle.create_image_view(&create_info, None).unwrap() }
    }

    pub fn acquire_next_image_khr(
        &self,
        swapchain_khr: vk::SwapchainKHR,
        timeout: u64,
        semaphore: Option<vk::Semaphore>,
        fence: Option<vk::Fence>,
    ) -> u32 {
        unsafe {
            self.handle
                .acquire_next_image_khr(swapchain_khr, timeout, semaphore, fence)
                .unwrap()
        }
    }

    pub fn queue_present(
        &self,
        queue: vk::Queue,
        swapchains: &[vk::SwapchainKHR],
        image_indices: &[u32],
        wait_semaphores: &[vk::Semaphore],
    ) {
        let present_info = vk::PresentInfoKHRBuilder::new()
            .swapchains(swapchains)
            .image_indices(image_indices)
            .wait_semaphores(wait_semaphores);
        unsafe { self.handle.queue_present_khr(queue, &present_info).unwrap() }
    }

    pub fn create_command_pool(
        &self,
        queue_family_index: u32,
        flags: vk::CommandPoolCreateFlags,
    ) -> vk::CommandPool {
        let create_info = vk::CommandPoolCreateInfoBuilder::new()
            .queue_family_index(queue_family_index)
            .flags(flags);
        unsafe { self.handle.create_command_pool(&create_info, None).unwrap() }
    }

    pub fn allocate_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        level: vk::CommandBufferLevel,
        count: u32,
    ) -> Vec<vk::CommandBuffer> {
        let create_info = &vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(command_pool)
            .level(level)
            .command_buffer_count(count);
        unsafe { self.handle.allocate_command_buffers(&create_info).unwrap() }
    }

    pub fn create_fence(&self) -> vk::Fence {
        let create_info = vk::FenceCreateInfoBuilder::new().flags(vk::FenceCreateFlags::SIGNALED);
        unsafe { self.handle.create_fence(&create_info, None).unwrap() }
    }

    pub fn create_semaphore(&self) -> vk::Semaphore {
        let create_info = vk::SemaphoreCreateInfo::default();
        unsafe { self.handle.create_semaphore(&create_info, None).unwrap() }
    }

    pub fn create_image(&self, image_info: ImageInfo) -> Image {
        let create_info = vk::ImageCreateInfoBuilder::new()
            .image_type(image_info.extent.into())
            .extent(image_info.extent.into())
            .format(image_info.format)
            .mip_levels(image_info.mip_levels)
            .samples(image_info.samples)
            .array_layers(image_info.array_layers)
            .tiling(image_info.tiling)
            .usage(image_info.usage);
        let image = unsafe { self.handle.create_image(&create_info, None).unwrap() };

        let mem_reqs = unsafe { self.handle.get_image_memory_requirements(image) };
        let mem_block = unsafe {
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
        };

        unsafe {
            self.handle
                .bind_image_memory(image, *mem_block.memory(), mem_block.offset())
                .unwrap();
        }

        Image {
            handle: image,
            info: image_info,
            memory: ImageMemory::DeviceImage {
                memory_block: mem_block,
            },
        }
    }
}

fn create_instance(entry: &EntryLoader) -> InstanceLoader {
    let app_name = CString::new("Rdx").unwrap();
    let engine_name = CString::new("Vulkan Engine").unwrap();
    let app_info = vk::ApplicationInfoBuilder::new()
        .api_version(vk::API_VERSION_1_2)
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .application_name(&app_name)
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(&engine_name);

    let mut instance_extensions = enumerate_required_surface_extensions();

    if cfg!(debug_assertions) {
        instance_extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION_NAME);
    }

    let mut instance_layers = Vec::new();
    if cfg!(debug_assertions) {
        instance_layers.push(VALIDATION_LAYER);
    }

    let instance_info = vk::InstanceCreateInfoBuilder::new()
        .application_info(&app_info)
        .enabled_extension_names(&instance_extensions)
        .enabled_layer_names(&instance_layers);

    unsafe { InstanceLoader::new(&entry, &instance_info, None).unwrap() }
}

fn enumerate_required_surface_extensions() -> Vec<*const c_char> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_wayland_surface::KHR_WAYLAND_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_xlib_surface::KHR_XLIB_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_xcb_surface::KHR_XCB_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(target_os = "android"))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_android_surface::KHR_ANDROID_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(target_os = "macos"))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::ext_metal_surface::EXT_METAL_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(target_os = "ios"))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::ext_metal_surface::EXT_METAL_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(target_os = "windows")]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_win32_surface::KHR_WIN32_SURFACE_EXTENSION_NAME,
    ];

    extensions
}

fn pick_physical_device(instance: &InstanceLoader) -> vk::PhysicalDevice {
    let physical_devices = unsafe { instance.enumerate_physical_devices(None).unwrap() };
    physical_devices[0]
}
