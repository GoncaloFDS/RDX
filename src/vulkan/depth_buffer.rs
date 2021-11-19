use crate::vulkan::command_buffers::CommandBuffers;
use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::image_view::ImageView;
use erupt::vk;
use std::rc::Rc;

pub struct DepthBuffer {
    image: Image,
    image_view: ImageView,
    format: vk::Format,
}

impl DepthBuffer {
    pub fn image_view(&self) -> &ImageView {
        &self.image_view
    }

    pub fn get_format(&self) -> vk::Format {
        self.format
    }

    pub fn new(device: Rc<Device>, command_buffers: &CommandBuffers, extent: vk::Extent2D) -> Self {
        let format = find_depth_format(&device);
        let mut image = Image::new(
            device.clone(),
            extent,
            format,
            Some(vk::ImageTiling::OPTIMAL),
            Some(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
        );
        image.allocate_memory();
        let image_view =
            ImageView::new(device.clone(), *image, format, vk::ImageAspectFlags::DEPTH);

        image.transition_image_layout(
            command_buffers,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        );

        DepthBuffer {
            image,
            image_view,
            format,
        }
    }

    pub fn has_stencil_component(format: vk::Format) -> bool {
        match format {
            vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT => true,
            _ => false,
        }
    }
}

fn find_depth_format(device: &Device) -> vk::Format {
    find_supported_format(
        device,
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

fn find_supported_format(
    device: &Device,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    *candidates
        .iter()
        .find(|format| {
            let format_properties = unsafe {
                device
                    .instance()
                    .get_physical_device_format_properties(device.physical_device(), **format)
            };

            match tiling {
                vk::ImageTiling::LINEAR => {
                    format_properties.linear_tiling_features.contains(features)
                }
                vk::ImageTiling::OPTIMAL => {
                    format_properties.optimal_tiling_features.contains(features)
                }
                _ => false,
            }
        })
        .unwrap()
}
