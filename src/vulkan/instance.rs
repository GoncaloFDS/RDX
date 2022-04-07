use crate::vulkan::surface::Surface;
use erupt::{vk, EntryLoader, InstanceLoader};
use erupt_bootstrap::{DebugMessenger, InstanceBuilder, InstanceMetadata, ValidationLayers};
use winit::window::Window;

pub struct Instance {
    handle: InstanceLoader,
    metadata: InstanceMetadata,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    surface: Surface,
    _entry: EntryLoader,
}

impl Instance {
    pub fn new(window: &Window) -> Self {
        let entry = EntryLoader::new().unwrap();
        let instance_builder = InstanceBuilder::new()
            .validation_layers(ValidationLayers::Request)
            .request_debug_messenger(DebugMessenger::Default)
            .require_api_version(1, 3)
            .require_surface_extensions(window)
            .unwrap();

        let (instance, debug_messenger, instance_metadata) =
            unsafe { instance_builder.build(&entry).unwrap() };

        let surface = Surface::new(&instance, &window);

        Instance {
            handle: instance,
            metadata: instance_metadata,
            debug_messenger,
            surface,
            _entry: entry,
        }
    }

    pub fn destroy(&self) {
        unsafe {
            if let Some(debug_messenger) = self.debug_messenger {
                self.handle
                    .destroy_debug_utils_messenger_ext(debug_messenger, None);
            }
            self.surface.destroy(self.handle());
            self.handle.destroy_instance(None);
        }
    }

    pub fn handle(&self) -> &InstanceLoader {
        &self.handle
    }

    pub fn metadata(&self) -> &InstanceMetadata {
        &self.metadata
    }

    pub fn surface(&self) -> Surface {
        self.surface
    }
}
