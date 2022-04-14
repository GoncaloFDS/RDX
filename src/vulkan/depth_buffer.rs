use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::instance::Instance;
use erupt::vk;

pub struct DepthBuffer {
    image: Image,
    format: vk::Format,
}

impl DepthBuffer {
    pub fn new(device: &mut Device, instance: &Instance, extent: vk::Extent2D) -> Self {
        let format = find_depth_format(device, instance);
        let mut image = Image::new(
            device,
            extent,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::ImageAspectFlags::DEPTH,
        );

        image.transition_image_layout(device, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        DepthBuffer { image, format }
    }

    pub fn image_view(&self) -> vk::ImageView {
        self.image.view()
    }

    pub fn get_format(&self) -> vk::Format {
        self.format
    }

    pub fn has_stencil_component(format: vk::Format) -> bool {
        matches!(
            format,
            vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT
        )
    }
}

fn find_depth_format(device: &Device, instance: &Instance) -> vk::Format {
    find_supported_format(
        device,
        instance,
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
    instance: &Instance,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    *candidates
        .iter()
        .find(|format| {
            let format_properties = unsafe {
                instance.handle().get_physical_device_format_properties(
                    device.metadata().physical_device(),
                    **format,
                )
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
