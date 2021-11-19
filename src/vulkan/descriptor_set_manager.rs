use crate::vulkan::descriptor_binding::DescriptorBinding;
use crate::vulkan::descriptor_set_layout::DescriptorSetLayout;
use crate::vulkan::device::Device;
use erupt::{vk, ExtendableFromConst};
use std::collections::HashMap;
use std::rc::Rc;

pub struct DescriptorSetManager {
    device: Rc<Device>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_set_layout: DescriptorSetLayout,
    binding_types: HashMap<u32, vk::DescriptorType>,
}

impl DescriptorSetManager {
    pub fn new(
        device: Rc<Device>,
        descriptor_bindings: &[DescriptorBinding],
        max_sets: usize,
    ) -> Self {
        let descriptor_set_layout = DescriptorSetLayout::new(device.clone(), descriptor_bindings);

        let pool_sizes = descriptor_bindings
            .iter()
            .map(|binding| {
                vk::DescriptorPoolSizeBuilder::new()
                    ._type(binding.descriptor_type)
                    .descriptor_count(binding.descriptor_count * max_sets as u32)
            })
            .collect::<Vec<_>>();

        let create_info = vk::DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(&pool_sizes)
            .max_sets(max_sets as _);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&create_info, None).unwrap() };

        let mut layouts = Vec::new();
        layouts.resize(max_sets as _, descriptor_set_layout.handle());

        let alloc_info = vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap() };

        let mut binding_types = HashMap::new();
        for binding in descriptor_bindings {
            binding_types.insert(binding.binding, binding.descriptor_type);
        }

        DescriptorSetManager {
            device,
            descriptor_pool,
            descriptor_sets,
            descriptor_set_layout,
            binding_types,
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
        vk::WriteDescriptorSetBuilder::new()
            .dst_set(self.descriptor_sets[index as usize])
            .dst_binding(binding)
            .dst_array_element(0)
            .descriptor_type(*self.binding_types.get(&binding).unwrap())
            .extend_from(&mut *acceleration_structures)
    }

    pub fn update_descriptors(&self, descriptor_writes: &[vk::WriteDescriptorSetBuilder<'_>]) {
        unsafe { self.device.update_descriptor_sets(descriptor_writes, &[]) }
    }
}

impl Drop for DescriptorSetManager {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_descriptor_pool(Some(self.descriptor_pool), None);
        }
    }
}
