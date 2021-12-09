use std::error::Error;

use spirv_builder::{Capability, MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("./shaders/raster", "spirv-unknown-vulkan1.2")
        .capability(Capability::RayTracingKHR)
        .extension("SPV_KHR_ray_tracing")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    SpirvBuilder::new("./shaders/raytracing", "spirv-unknown-spv1.3")
        .capability(Capability::RayTracingKHR)
        .capability(Capability::RuntimeDescriptorArray)
        .extension("SPV_KHR_ray_tracing")
        .extension("SPV_EXT_descriptor_indexing")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    SpirvBuilder::new("./shaders/ui", "spirv-unknown-vulkan1.2")
        .capability(Capability::RayTracingKHR)
        .extension("SPV_KHR_ray_tracing")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
