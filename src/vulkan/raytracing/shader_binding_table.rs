use crate::vulkan::buffer::Buffer;
use crate::vulkan::device::Device;
use crate::vulkan::raytracing::raytracing_pipeline::RaytracingPipeline;
use crate::vulkan::raytracing::raytracing_properties::RaytracingProperties;
use erupt::vk;

pub struct Entry {
    group_index: u32,
}

impl Entry {
    pub fn new(group_index: u32) -> Self {
        Entry { group_index }
    }
}

#[derive(Debug, Default)]
struct ShaderInfo {
    stride: u32,
    offset: u32,
    size: u32,
}

impl ShaderInfo {
    pub fn new(stride: u32, offset: u32, size: u32) -> Self {
        ShaderInfo {
            stride,
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
    pub fn new(
        device: &mut Device,
        raytracing_pipeline: &RaytracingPipeline,
        raytracing_properties: &RaytracingProperties,
        raygen_groups: &[Entry],
        miss_groups: &[Entry],
        hit_groups: &[Entry],
    ) -> Self {
        let handle_size_aligned = aligned_size(raytracing_properties);
        let raygen_stride = handle_size_aligned;
        let miss_stride = handle_size_aligned;
        let hit_stride = handle_size_aligned;

        let raygen_info =
            ShaderInfo::new(raygen_stride, 0, raygen_groups.len() as u32 * raygen_stride);

        let miss_info = ShaderInfo::new(
            miss_stride,
            raygen_info.size,
            miss_groups.len() as u32 * miss_stride,
        );

        let hit_info = ShaderInfo::new(
            hit_stride,
            raygen_info.size + miss_info.stride,
            hit_groups.len() as u32 * hit_stride,
        );

        let sbt_size = raygen_info.size + miss_info.size + hit_info.size;

        let mut buffer = Buffer::empty(
            device,
            sbt_size as u64,
            vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS | vk::BufferUsageFlags::TRANSFER_SRC,
            gpu_alloc::UsageFlags::DEVICE_ADDRESS | gpu_alloc::UsageFlags::HOST_ACCESS,
        );

        let handle_size = raytracing_properties.shader_group_handle_size();
        let group_count = (raygen_groups.len() + miss_groups.len() + hit_groups.len()) as u32;
        let mut shader_handle_storage = vec![0u8; (handle_size * group_count) as usize];
        let mut table_data = vec![0u8; (handle_size_aligned * group_count) as usize];

        unsafe {
            device
                .handle()
                .get_ray_tracing_shader_group_handles_khr(
                    raytracing_pipeline.handle(),
                    0,
                    group_count,
                    shader_handle_storage.len(),
                    shader_handle_storage.as_mut_ptr() as _,
                )
                .unwrap()
        };

        for i in 0..group_count {
            let a = (i * handle_size_aligned) as usize;
            let b = (i * handle_size_aligned + handle_size) as usize;
            let c = (i * handle_size) as usize;
            let d = (i * handle_size + handle_size) as usize;
            table_data[a..b].copy_from_slice(&shader_handle_storage[c..d]);
        }

        buffer.write_data(device, &table_data, 0);

        ShaderBindingTable {
            raygen_info,
            miss_info,
            hit_info,
            buffer,
        }
    }

    pub fn raygen_device_address(&self, device: &Device) -> vk::DeviceAddress {
        self.buffer.get_device_address(device) + self.raygen_info.offset as vk::DeviceAddress
    }

    pub fn miss_device_address(&self, device: &Device) -> vk::DeviceAddress {
        self.buffer.get_device_address(device) + self.miss_info.offset as vk::DeviceAddress
    }

    pub fn hit_device_address(&self, device: &Device) -> vk::DeviceAddress {
        self.buffer.get_device_address(device) + self.hit_info.offset as vk::DeviceAddress
    }

    pub fn raygen_stride(&self) -> u32 {
        self.raygen_info.stride
    }

    pub fn miss_stride(&self) -> u32 {
        self.miss_info.stride
    }

    pub fn hit_stride(&self) -> u32 {
        self.hit_info.stride
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

    pub fn raygen_device_region(&self, device: &Device) -> vk::StridedDeviceAddressRegionKHR {
        vk::StridedDeviceAddressRegionKHRBuilder::new()
            .device_address(self.raygen_device_address(device))
            .stride(self.raygen_stride() as _)
            .size(self.raygen_size() as _)
            .build()
    }

    pub fn miss_device_region(&self, device: &Device) -> vk::StridedDeviceAddressRegionKHR {
        vk::StridedDeviceAddressRegionKHRBuilder::new()
            .device_address(self.miss_device_address(device))
            .stride(self.miss_stride() as _)
            .size(self.miss_size() as _)
            .build()
    }

    pub fn hit_device_region(&self, device: &Device) -> vk::StridedDeviceAddressRegionKHR {
        vk::StridedDeviceAddressRegionKHRBuilder::new()
            .device_address(self.hit_device_address(device))
            .stride(self.hit_stride() as _)
            .size(self.hit_size() as _)
            .build()
    }

    pub fn callable_device_region(&self, _device: &Device) -> vk::StridedDeviceAddressRegionKHR {
        vk::StridedDeviceAddressRegionKHRBuilder::default().build()
    }
}

fn round_up(size: u32, alignment: u32) -> u32 {
    ((size + alignment - 1) / alignment) * alignment
}

fn aligned_size(properties: &RaytracingProperties) -> u32 {
    round_up(
        properties.shader_group_handle_size(),
        properties.shader_group_base_alignment(),
    )
}
