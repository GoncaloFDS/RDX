use erupt::vk;

pub struct SubresourceRange;

impl SubresourceRange {
    pub fn with_aspect(image_aspect_flags: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask: image_aspect_flags,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        }
    }
}
