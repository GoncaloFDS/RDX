use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::surface::Surface;
use erupt::vk;
use std::rc::Rc;
use winit::window::Window;

pub struct SwapchainImages {
    pub image: Image,
    pub view: ImageView,
}

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
    images: Vec<SwapchainImages>,
    support_details: SwapchainSupportDetails,
}

impl Swapchain {}

impl Swapchain {
    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn swapchain_images(&self) -> &[SwapchainImages] {
        &self.images
    }

    pub fn new(device: Rc<Device>, window: &Window) -> Self {
        let surface = Surface::new(device.clone(), window);
        let extent = vk::Extent2D {
            width: window.inner_size().width,
            height: window.inner_size().height,
        };

        let queue_props = unsafe {
            &device
                .get_instance()
                .get_physical_device_queue_family_properties(device.get_physical_device(), None)
        };

        let (i, _) = queue_props
            .iter()
            .enumerate()
            .find(|(i, properties)| {
                let supports_present = unsafe {
                    device
                        .get_instance()
                        .get_physical_device_surface_support_khr(
                            device.get_physical_device(),
                            *i as u32,
                            *surface,
                        )
                        .unwrap()
                };
                let supports_graphics = properties.queue_flags.contains(vk::QueueFlags::GRAPHICS);

                supports_present && supports_graphics
            })
            .expect("Failed to find a Queue that supports both present and graphics");

        let surface_formats = unsafe {
            device
                .get_instance()
                .get_physical_device_surface_formats_khr(
                    device.get_physical_device(),
                    *surface,
                    None,
                )
                .unwrap()
        };

        let color_format;
        let color_space;
        // If the surface format list only includes one entry with VK_FORMAT_UNDEFINED,
        // there is no preferred format, so we assume VK_FORMAT_B8G8R8A8_UNORM
        if surface_formats.len() == 1 && surface_formats[0].format == vk::Format::UNDEFINED {
            color_format = vk::Format::B8G8R8A8_UNORM;
            color_space = surface_formats[0].color_space;
        } else {
            match surface_formats
                .iter()
                .find(|surface_format| surface_format.format == vk::Format::B8G8R8A8_UNORM)
            {
                None => {
                    color_format = surface_formats[0].format;
                    color_space = surface_formats[0].color_space;
                }
                Some(surface_format) => {
                    color_format = surface_format.format;
                    color_space = surface_format.color_space;
                }
            }
        }

        log::info!("{:?}, {:?}", color_format, color_space);
        Swapchain {
            device,
            handle: Default::default(),
            surface,
            format: color_format,
            color_space,
            extent,
            images: vec![],
            support_details: Default::default(),
        }
    }
}
