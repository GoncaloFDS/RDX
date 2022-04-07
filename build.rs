use std::error::Error;

use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    SpirvBuilder::new("./shaders/raster", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    SpirvBuilder::new("./shaders/ui", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
