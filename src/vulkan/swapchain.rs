use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::surface::Surface;
use erupt::extensions::khr_surface::SurfaceCapabilitiesKHR;
use erupt::vk;
use std::rc::Rc;
use winit::window::Window;

#[derive(Default)]
pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct Swapchain {
    device: Rc<Device>,
    handle: vk::SwapchainKHR,
    surface: Surface,
    format: vk::Format,
    color_space: vk::ColorSpaceKHR,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    image_views: Vec<ImageView>,
    support_details: SwapchainSupportDetails,
}

impl Swapchain {
    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn images(&self) -> &[vk::Image] {
        &self.images
    }

    pub fn image_views(&self) -> &Vec<ImageView> {
        &self.image_views
    }

    pub fn new(device: Rc<Device>, window: &Window, present_mode: vk::PresentModeKHR) -> Self {
        let surface = Surface::new(device.clone(), window);

        let support_details = Self::query_swapchain_support(&device, &surface);

        let surface_format = Self::choose_surface_format(&support_details.formats);
        let present_mode = Self::choose_present_mode(present_mode, &support_details.present_modes);
        let extent = Self::choose_extent(window, &support_details.capabilities);
        let image_count = Self::choose_image_count(&support_details.capabilities);

        let create_info = vk::SwapchainCreateInfoKHRBuilder::new()
            .surface(surface.handle())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .pre_transform(support_details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null())
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE);

        let swapchain = unsafe { device.create_swapchain_khr(&create_info, None).unwrap() };

        let format = surface_format.format;
        let color_space = surface_format.color_space;

        let images = unsafe { device.get_swapchain_images_khr(swapchain, None).unwrap() };
        let image_views = images
            .iter()
            .map(|image| {
                ImageView::new(device.clone(), *image, format, vk::ImageAspectFlags::COLOR)
            })
            .collect::<Vec<_>>();

        Swapchain {
            device,
            handle: swapchain,
            surface,
            format,
            color_space,
            extent,
            images,
            image_views,
            support_details,
        }
    }

    fn query_swapchain_support(device: &Device, surface: &Surface) -> SwapchainSupportDetails {
        let supports_surface = unsafe {
            device
                .instance()
                .get_physical_device_surface_support_khr(
                    device.physical_device(),
                    device.graphics_family_index(),
                    surface.handle(),
                )
                .unwrap()
        };

        if !supports_surface {
            log::error!("physical device does not support this surface");
            panic!();
        }

        let capabilities = unsafe {
            device
                .instance()
                .get_physical_device_surface_capabilities_khr(
                    device.physical_device(),
                    surface.handle(),
                )
                .unwrap()
        };

        let formats = unsafe {
            device
                .instance()
                .get_physical_device_surface_formats_khr(
                    device.physical_device(),
                    surface.handle(),
                    None,
                )
                .unwrap()
        };

        let present_modes = unsafe {
            device
                .instance()
                .get_physical_device_surface_present_modes_khr(
                    device.physical_device(),
                    surface.handle(),
                    None,
                )
                .unwrap()
        };

        SwapchainSupportDetails {
            capabilities,
            formats,
            present_modes,
        }
    }

    fn choose_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        *formats
            .iter()
            .find(|format| {
                format.format == vk::Format::B8G8R8A8_UNORM
                    && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR
            })
            .unwrap()
    }

    fn choose_present_mode(
        requested_mode: vk::PresentModeKHR,
        present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        if present_modes.contains(&requested_mode) {
            log::debug!("Using {:?} present mode", requested_mode);
            requested_mode
        } else {
            log::debug!("Using fallback present mode");
            vk::PresentModeKHR::FIFO_KHR
        }
    }

    fn choose_extent(window: &Window, _capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        }
    }

    fn choose_image_count(_capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
        3
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_swapchain_khr(Some(self.handle), None);
        }
    }
}
