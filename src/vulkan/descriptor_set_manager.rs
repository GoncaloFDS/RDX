use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::descriptor_set_layout::DescriptorSetLayout;
use crate::vulkan::device::Device;
use erupt::{vk, ExtendableFrom, SmallVec};
use std::collections::HashMap;

pub struct DescriptorSetManager {
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: SmallVec<vk::DescriptorSet>,
    descriptor_set_layout: DescriptorSetLayout,
    binding_types: HashMap<u32, vk::DescriptorType>,
}

impl DescriptorSetManager {
    pub fn descriptor_set_layout(&self) -> &DescriptorSetLayout {
        &self.descriptor_set_layout
    }

    pub fn descriptor_set(&self, index: usize) -> vk::DescriptorSet {
        self.descriptor_sets[index]
    }

    pub fn new(device: &Device, descriptor_bindings: &[DescriptorBinding], max_sets: u32) -> Self {
        let descriptor_set_layout = DescriptorSetLayout::new(&device, descriptor_bindings);

        let pool_sizes = descriptor_bindings
            .iter()
            .map(|binding| {
                vk::DescriptorPoolSizeBuilder::new()
                    ._type(binding.descriptor_type)
                    .descriptor_count(binding.descriptor_count * max_sets)
            })
            .collect::<Vec<_>>();

        let create_info = vk::DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(&pool_sizes)
            .max_sets(max_sets as _);

        let descriptor_pool = unsafe {
            device
                .handle()
                .create_descriptor_pool(&create_info, None)
                .unwrap()
        };

        let mut layouts = Vec::new();
        layouts.resize(max_sets as _, descriptor_set_layout.handle());

        let alloc_info = vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe {
            device
                .handle()
                .allocate_descriptor_sets(&alloc_info)
                .unwrap()
        };

        let mut binding_types = HashMap::new();
        for binding in descriptor_bindings {
            binding_types.insert(binding.binding, binding.descriptor_type);
        }

        DescriptorSetManager {
            descriptor_pool,
            descriptor_sets,
            descriptor_set_layout,
            binding_types,
        }
    }

    pub fn destroy(&self, device: &Device) {
        self.descriptor_set_layout.destroy(device);
        unsafe {
            device
                .handle()
                .destroy_descriptor_pool(self.descriptor_pool, None)
        }
    }

    pub fn bind_buffer<'a>(
        &self,
        index: u32,
        binding: u32,
        buffer_info: &'a [vk::DescriptorBufferInfoBuilder<'a>],
    ) -> vk::WriteDescriptorSetBuilder<'a> {
        vk::WriteDescriptorSetBuilder::new()
            .dst_set(self.descriptor_sets[index as usize])
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(*self.binding_types.get(&binding).unwrap())
            .buffer_info(buffer_info)
    }

    pub fn bind_image<'a>(
        &self,
        index: u32,
        binding: u32,
        image_info: &'a [vk::DescriptorImageInfoBuilder<'a>],
    ) -> vk::WriteDescriptorSetBuilder<'a> {
        vk::WriteDescriptorSetBuilder::new()
            .dst_set(self.descriptor_sets[index as usize])
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(*self.binding_types.get(&binding).unwrap())
            .image_info(image_info)
    }

    pub fn bind_acceleration_structure<'a>(
        &self,
        index: u32,
        binding: u32,
        acceleration_structures: &'a mut vk::WriteDescriptorSetAccelerationStructureKHRBuilder<'a>,
    ) -> vk::WriteDescriptorSetBuilder<'a> {
        let mut write = vk::WriteDescriptorSetBuilder::new()
            .dst_set(self.descriptor_sets[index as usize])
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(*self.binding_types.get(&binding).unwrap())
            .extend_from(acceleration_structures);
        write.descriptor_count = 1;
        write
    }

    pub fn update_descriptors(
        &self,
        device: &Device,
        descriptor_writes: &[vk::WriteDescriptorSetBuilder<'_>],
    ) {
        unsafe {
            device
                .handle()
                .update_descriptor_sets(descriptor_writes, &[])
        }
    }
}
