use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use crate::vulkan::raytracing::raytracing_pipeline::RaytracingPipeline;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use erupt::vk;
use std::rc::Rc;

pub struct Entry {
    group_index: u32,
}

impl Entry {
    pub fn new(group_index: u32) -> Self {
        Entry { group_index }
    }
}

#[derive(Default)]
struct ShaderInfo {
    entry_size: u32,
    offset: u32,
    size: u32,
}

impl ShaderInfo {
    pub fn new(entry_size: u32, offset: u32, size: u32) -> Self {
        ShaderInfo {
            entry_size,
            offset,
            size,
        }
    }
}

pub struct ShaderBindingTable {
    raygen_info: ShaderInfo,
    miss_info: ShaderInfo,
    hit_info: ShaderInfo,
    buffer: Buffer,
}

impl ShaderBindingTable {
    pub fn uninitialized(device: Rc<Device>) -> Self {
        ShaderBindingTable {
            raygen_info: Default::default(),
            miss_info: Default::default(),
            hit_info: Default::default(),
            buffer: Buffer::uninitialized(device),
        }
    }

    pub fn new(
        device: Rc<Device>,
        raytracing_pipeline: &RaytracingPipeline,
        raytracing_properties: &RaytracingProperties,
        raygen_groups: &[Entry],
        miss_groups: &[Entry],
        hit_groups: &[Entry],
    ) -> Self {
        let raygen_entry_size = get_entry_size(raytracing_properties);
        let miss_entry_size = get_entry_size(raytracing_properties);
        let hit_entry_size = get_entry_size(raytracing_properties);

        let raygen_info = ShaderInfo::new(
            raygen_entry_size,
            0,
            raygen_groups.len() as u32 * raygen_entry_size,
        );

        let miss_info = ShaderInfo::new(
            miss_entry_size,
            raygen_info.size,
            miss_groups.len() as u32 * miss_entry_size,
        );

        let hit_info = ShaderInfo::new(
            hit_entry_size,
            raygen_info.size + miss_info.entry_size,
            hit_groups.len() as u32 * hit_entry_size,
        );

        let sbt_size = raygen_info.size + miss_info.size + hit_info.size;

        let mut buffer = Buffer::new(
            device.clone(),
            sbt_size as u64,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::TRANSFER_SRC,
        );
        buffer.allocate_memory(
            gpu_alloc::UsageFlags::DEVICE_ADDRESS | gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        let handle_size = raytracing_properties.shader_group_handle_size();
        let group_count = (raygen_groups.len() + miss_groups.len() + hit_groups.len()) as u32;

        let mut shader_handle_storage = vec![0u8; (handle_size * group_count) as usize];

        unsafe {
            device
                .get_ray_tracing_shader_group_handles_khr(
                    raytracing_pipeline.handle(),
                    0,
                    group_count,
                    shader_handle_storage.len(),
                    shader_handle_storage.as_mut_ptr() as _,
                )
                .unwrap()
        };

        buffer.write_data(&shader_handle_storage, 0);

        ShaderBindingTable {
            raygen_info,
            miss_info,
            hit_info,
            buffer,
        }
    }

    pub fn raygen_device_address(&self) -> vk::DeviceAddress {
        self.buffer.get_device_address() + self.raygen_info.offset as vk::DeviceAddress
    }

    pub fn miss_device_address(&self) -> vk::DeviceAddress {
        self.buffer.get_device_address() + self.miss_info.offset as vk::DeviceAddress
    }

    pub fn hit_device_address(&self) -> vk::DeviceAddress {
        self.buffer.get_device_address() + self.hit_info.offset as vk::DeviceAddress
    }

    pub fn raygen_entry_size(&self) -> u32 {
        self.raygen_info.entry_size
    }

    pub fn miss_entry_size(&self) -> u32 {
        self.miss_info.entry_size
    }

    pub fn hit_entry_size(&self) -> u32 {
        self.hit_info.entry_size
    }

    pub fn raygen_size(&self) -> u32 {
        self.raygen_info.size
    }

    pub fn miss_size(&self) -> u32 {
        self.miss_info.size
    }

    pub fn hit_size(&self) -> u32 {
        self.hit_info.size
    }

    pub fn raygen_device_region(&self) -> vk::StridedDeviceAddressRegionKHR {
        vk::StridedDeviceAddressRegionKHRBuilder::new()
            .device_address(self.raygen_device_address())
            .stride(self.raygen_entry_size() as _)
            .size(self.raygen_size() as _)
            .build()
    }

    pub fn miss_device_region(&self) -> vk::StridedDeviceAddressRegionKHR {
        vk::StridedDeviceAddressRegionKHRBuilder::new()
            .device_address(self.miss_device_address())
            .stride(self.miss_entry_size() as _)
            .size(self.miss_size() as _)
            .build()
    }

    pub fn hit_device_region(&self) -> vk::StridedDeviceAddressRegionKHR {
        vk::StridedDeviceAddressRegionKHRBuilder::new()
            .device_address(self.hit_device_address())
            .stride(self.hit_entry_size() as _)
            .size(self.hit_size() as _)
            .build()
    }

    pub fn callable_device_region(&self) -> vk::StridedDeviceAddressRegionKHR {
        vk::StridedDeviceAddressRegionKHRBuilder::default().build()
    }
}

fn round_up(size: u32, alignment: u32) -> u32 {
    ((size + alignment - 1) / alignment) * alignment
}

fn get_entry_size(properties: &RaytracingProperties) -> u32 {
    round_up(
        properties.shader_group_handle_size(),
        properties.shader_group_base_alignment(),
    )
}
