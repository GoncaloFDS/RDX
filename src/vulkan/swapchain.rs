use crate::vulkan::device::Device;
use crate::vulkan::instance::Instance;
use crate::vulkan::surface::Surface;
use erupt::utils::VulkanResult;
use erupt::vk;
use erupt_bootstrap::AcquiredFrame;
use winit::window::Window;

const FORMAT_CANDIDATES: &[vk::Format] = &[
    vk::Format::R8G8B8A8_UNORM,
    vk::Format::B8G8R8A8_UNORM,
    vk::Format::A8B8G8R8_UNORM_PACK32,
];

pub struct Swapchain {
    handle: erupt_bootstrap::Swapchain,
    surface_format: vk::SurfaceFormatKHR,
}

impl Swapchain {
    pub fn new(window: &Window, instance: &Instance, surface: Surface, device: &Device) -> Self {
        let surface_formats = unsafe {
            instance
                .handle()
                .get_physical_device_surface_formats_khr(
                    device.metadata().physical_device(),
                    surface.handle(),
                    None,
                )
                .unwrap()
        };
        let surface_format = match *surface_formats.as_slice() {
            [single] if single.format == vk::Format::UNDEFINED => vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: single.color_space,
            },
            _ => *surface_formats
                .iter()
                .find(|surface_format| FORMAT_CANDIDATES.contains(&surface_format.format))
                .unwrap_or(&surface_formats[0]),
        };

        let mut swapchain_options = erupt_bootstrap::SwapchainOptions::default();
        swapchain_options.format_preference(&[surface_format]);

        let size = window.inner_size();
        let swapchain = erupt_bootstrap::Swapchain::new(
            swapchain_options,
            surface.handle(),
            device.metadata().physical_device(),
            device.handle(),
            vk::Extent2D {
                width: size.width,
                height: size.height,
            },
        );

        Swapchain {
            handle: swapchain,
            surface_format,
        }
    }

    pub fn surface_format(&self) -> vk::SurfaceFormatKHR {
        self.surface_format
    }

    pub fn resize(&mut self, extent: vk::Extent2D) {
        self.handle.update(extent)
    }

    #[inline]
    pub fn frames_in_flight(&self) -> usize {
        self.handle.frames_in_flight()
    }

    #[inline]
    pub fn images(&self) -> &[vk::Image] {
        self.handle.images()
    }

    #[inline]
    pub fn format(&self) -> vk::SurfaceFormatKHR {
        self.handle.format()
    }

    #[inline]
    pub fn extent(&self) -> vk::Extent2D {
        self.handle.extent()
    }

    #[inline]
    pub fn acquire(
        &mut self,
        instance: &Instance,
        device: &Device,
        timeout_ns: u64,
    ) -> VulkanResult<AcquiredFrame> {
        unsafe {
            self.handle
                .acquire(instance.handle(), device.handle(), timeout_ns)
        }
    }

    #[inline]
    pub fn queue_present(
        &mut self,
        device: &Device,
        queue: vk::Queue,
        render_complete: vk::Semaphore,
        image_index: usize,
    ) -> VulkanResult<()> {
        unsafe {
            self.handle
                .queue_present(device.handle(), queue, render_complete, image_index)
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            self.handle.destroy(device.handle());
        }
    }
}
