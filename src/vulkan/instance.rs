use crate::vulkan::debug_utils;
use erupt::extensions::ext_debug_utils;
use erupt::{vk, EntryLoader, InstanceLoader};
use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::c_char;

pub struct Instance {
    handle: InstanceLoader,
    _entry: EntryLoader,
}

impl Deref for Instance {
    type Target = InstanceLoader;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Instance {
    pub fn handle(&self) -> &InstanceLoader {
        &self.handle
    }

    pub fn new() -> Self {
        let entry = EntryLoader::new().unwrap();
        let instance = create_instance(&entry);
        Instance {
            handle: instance,
            _entry: entry,
        }
    }
}

fn create_instance(entry: &EntryLoader) -> InstanceLoader {
    let app_name = CString::new("Rdx").unwrap();
    let engine_name = CString::new("Vulkan Engine").unwrap();
    let app_info = vk::ApplicationInfoBuilder::new()
        .api_version(vk::API_VERSION_1_2)
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .application_name(&app_name)
        .engine_version(vk::make_api_version(0, 1, 0, 0))
        .engine_name(&engine_name);

    let mut instance_extensions = enumerate_required_surface_extensions();

    if cfg!(debug_assertions) {
        instance_extensions.push(ext_debug_utils::EXT_DEBUG_UTILS_EXTENSION_NAME);
    }

    let mut instance_layers = Vec::new();
    if cfg!(debug_assertions) {
        instance_layers.push(debug_utils::VALIDATION_LAYER);
    }

    let instance_info = vk::InstanceCreateInfoBuilder::new()
        .application_info(&app_info)
        .enabled_extension_names(&instance_extensions)
        .enabled_layer_names(&instance_layers);

    unsafe { InstanceLoader::new(&entry, &instance_info, None).unwrap() }
}

fn enumerate_required_surface_extensions() -> Vec<*const c_char> {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_wayland_surface::KHR_WAYLAND_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_xlib_surface::KHR_XLIB_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_xcb_surface::KHR_XCB_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(target_os = "android"))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_android_surface::KHR_ANDROID_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(target_os = "macos"))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::ext_metal_surface::EXT_METAL_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(any(target_os = "ios"))]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::ext_metal_surface::EXT_METAL_SURFACE_EXTENSION_NAME,
    ];

    #[cfg(target_os = "windows")]
    let extensions = vec![
        erupt::extensions::khr_surface::KHR_SURFACE_EXTENSION_NAME,
        erupt::extensions::khr_win32_surface::KHR_WIN32_SURFACE_EXTENSION_NAME,
    ];

    extensions
}
