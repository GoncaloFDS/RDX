use crate::vulkan::device::Device;
use crate::vulkan::instance::Instance;
use crate::vulkan::surface::Surface;
use erupt::{vk, DeviceLoader};
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
    pub fn new(
        window: &Window,
        instance: &Instance,
        surface: Surface,
        device: &DeviceLoader,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        let surface_formats = unsafe {
            instance
                .handle()
                .get_physical_device_surface_formats_khr(physical_device, surface.handle(), None)
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
        swapchain_options.frames_in_flight(3);
        swapchain_options.format_preference(&[surface_format]);
        swapchain_options
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST);

        let size = window.inner_size();
        let swapchain = erupt_bootstrap::Swapchain::new(
            swapchain_options,
            surface.handle(),
            physical_device,
            device,
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
        device: &DeviceLoader,
        timeout_ns: u64,
    ) -> AcquiredFrame {
        unsafe {
            self.handle
                .acquire(instance.handle(), device, timeout_ns)
                .unwrap()
        }
    }

    #[inline]
    pub fn queue_present(
        &mut self,
        device: &DeviceLoader,
        queue: vk::Queue,
        render_complete: vk::Semaphore,
        image_index: usize,
    ) {
        unsafe {
            self.handle
                .queue_present(device, queue, render_complete, image_index)
                .unwrap();
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        unsafe {
            self.handle.destroy(device.handle());
        }
    }

    pub fn as_mut(&mut self) -> &mut erupt_bootstrap::Swapchain {
        &mut self.handle
    }
}
