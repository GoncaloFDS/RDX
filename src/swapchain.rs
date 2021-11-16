use crate::device::Device;
use erupt::extensions::{khr_surface, khr_swapchain};
use erupt::utils::surface;
use erupt::vk;
use std::rc::Rc;
use winit::window::Window;

#[derive(Default)]
pub struct SwapchainBuffer {
    pub image: vk::Image,
    pub view: vk::ImageView,
}

pub struct Swapchain {
    device: Rc<Device>,
    handle: khr_swapchain::SwapchainKHR,
    surface: khr_surface::SurfaceKHR,
    color_format: vk::Format,
    color_space: khr_surface::ColorSpaceKHR,
    images: Vec<vk::Image>,
    buffers: Vec<SwapchainBuffer>,
    queue_node_index: u32,
}

impl Swapchain {
    pub fn new(device: Rc<Device>) -> Self {
        Swapchain {
            device,
            handle: Default::default(),
            surface: Default::default(),
            color_format: Default::default(),
            color_space: Default::default(),
            images: vec![],
            buffers: vec![],
            queue_node_index: 0,
        }
    }

    pub fn init_with_window(&mut self, window: &Window) {
        self.surface =
            unsafe { surface::create_surface(&self.device.get_instance(), window, None).unwrap() };

        let queue_props = unsafe {
            &self
                .device
                .get_instance()
                .get_physical_device_queue_family_properties(
                    self.device.get_physical_device(),
                    None,
                )
        };

        let (i, _) = queue_props
            .iter()
            .enumerate()
            .find(|(i, properties)| {
                let supports_present = unsafe {
                    self.device
                        .get_instance()
                        .get_physical_device_surface_support_khr(
                            self.device.get_physical_device(),
                            *i as u32,
                            self.surface,
                        )
                        .unwrap()
                };
                let supports_graphics = properties.queue_flags.contains(vk::QueueFlags::GRAPHICS);

                supports_present && supports_graphics
            })
            .expect("Failed to find a Queue that supports both present and graphics");

        self.queue_node_index = i as _;

        let surface_formats = unsafe {
            self.device
                .get_instance()
                .get_physical_device_surface_formats_khr(
                    self.device.get_physical_device(),
                    self.surface,
                    None,
                )
                .unwrap()
        };

        // If the surface format list only includes one entry with VK_FORMAT_UNDEFINED,
        // there is no preferred format, so we assume VK_FORMAT_B8G8R8A8_UNORM
        if surface_formats.len() == 1 && surface_formats[0].format == vk::Format::UNDEFINED {
            self.color_format = vk::Format::B8G8R8A8_UNORM;
            self.color_space = surface_formats[0].color_space;
        } else {
            match surface_formats
                .iter()
                .find(|surface_format| surface_format.format == vk::Format::B8G8R8A8_UNORM)
            {
                None => {
                    self.color_format = surface_formats[0].format;
                    self.color_space = surface_formats[0].color_space;
                }
                Some(surface_format) => {
                    self.color_format = surface_format.format;
                    self.color_space = surface_format.color_space;
                }
            }
        }

        log::info!(
            "color format: {:?}, color space: {:?}",
            self.color_format,
            self.color_space
        );
    }

    pub fn create(&mut self, width: u32, height: u32, vsync: bool) {
        let old_swapchain = self.handle;

        let surface_caps = unsafe {
            self.device
                .get_instance()
                .get_physical_device_surface_capabilities_khr(
                    self.device.get_physical_device(),
                    self.surface,
                )
                .unwrap()
        };

        let present_modes = unsafe {
            self.device
                .get_instance()
                .get_physical_device_surface_present_modes_khr(
                    self.device.get_physical_device(),
                    self.surface,
                    None,
                )
                .unwrap()
        };

        let swapchain_extent = vk::Extent2D { width, height };

        let present_mode = if vsync {
            khr_surface::PresentModeKHR::FIFO_KHR
        } else {
            *present_modes
                .iter()
                .find(|mode| **mode == khr_surface::PresentModeKHR::MAILBOX_KHR)
                .unwrap_or(&khr_surface::PresentModeKHR::IMMEDIATE_KHR)
        };

        let mut desired_number_of_images = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0
            && desired_number_of_images > surface_caps.max_image_count
        {
            desired_number_of_images = surface_caps.max_image_count
        }

        let swapchain_create_info = khr_swapchain::SwapchainCreateInfoKHRBuilder::new()
            .surface(self.surface)
            .min_image_count(desired_number_of_images)
            .image_format(self.color_format)
            .image_color_space(self.color_space)
            .image_extent(swapchain_extent)
            .image_usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST,
            )
            .pre_transform(khr_surface::SurfaceTransformFlagBitsKHR::IDENTITY_KHR)
            .image_array_layers(1)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[])
            .present_mode(present_mode)
            .old_swapchain(old_swapchain)
            .clipped(true)
            .composite_alpha(khr_surface::CompositeAlphaFlagBitsKHR::OPAQUE_KHR);

        self.handle = unsafe { self.device.create_swapchain_khr(swapchain_create_info) };

        if !old_swapchain.is_null() {
            for buffer in &self.buffers {
                self.device.destroy_image_view(buffer.view);
            }
            self.device.destroy_swapchain_khr(old_swapchain);
        }

        self.images = self.device.get_swapchain_images_khr(self.handle);

        self.buffers
            .resize_with(self.images.len(), || SwapchainBuffer::default());
        for i in 0..self.images.len() {
            let color_attachment_view = vk::ImageViewCreateInfoBuilder::new()
                .format(self.color_format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .view_type(vk::ImageViewType::_2D)
                .flags(vk::ImageViewCreateFlags::empty())
                .image(self.images[i])
                .build();

            let image_view = self.device.create_image_view(color_attachment_view);

            self.buffers[i].image = self.images[i];
            self.buffers[i].view = image_view;
        }
    }

    pub fn acquire_next_image(&mut self, wait_semaphore: vk::Semaphore) -> u32 {
        self.device
            .acquire_next_image_khr(self.handle, u64::MAX, Some(wait_semaphore), None)
    }

    pub fn queue_present(
        &mut self,
        queue: vk::Queue,
        image_index: u32,
        wait_semaphore: vk::Semaphore,
    ) {
        self.device
            .queue_present(queue, &[self.handle], &[image_index], &[wait_semaphore])
    }
}
