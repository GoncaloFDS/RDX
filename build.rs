use std::error::Error;

use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("./resources/shaders/raster", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    SpirvBuilder::new("./resources/shaders/ui", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    SpirvBuilder::new("./resources/shaders/raytracing", "spirv-unknown-spv1.2")
        .capability(Capability::RayTracingKHR)
        .capability(Capability::RuntimeDescriptorArray)
        .extension("SPV_KHR_ray_tracing")
        .extension("SPV_EXT_descriptor_indexing")
        .extension("SPV_KHR_storage_buffer_storage_class")
        .extension("SPV_KHR_variable_pointers")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
