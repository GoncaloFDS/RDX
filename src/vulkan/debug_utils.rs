use crate::vulkan::device::Device;
use erupt::{cstr, vk};
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::rc::Rc;

pub const VALIDATION_LAYER: *const c_char = cstr!("VK_LAYER_KHRONOS_validation");

pub struct DebugMessenger {
    handle: vk::DebugUtilsMessengerEXT,
    device: Rc<Device>,
}

impl DebugMessenger {
    pub fn new(device: Rc<Device>) -> Self {
        let handle = if cfg!(debug_assertions) {
            log::debug!("Enabling debug messenger");
            let messenger_info = vk::DebugUtilsMessengerCreateInfoEXTBuilder::new()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::INFO_EXT
                        | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE_EXT
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT
                        | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL_EXT
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION_EXT
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE_EXT,
                )
                .pfn_user_callback(Some(debug_callback));

            unsafe {
                device
                    .instance()
                    .create_debug_utils_messenger_ext(&messenger_info, None)
                    .unwrap()
            }
        } else {
            vk::DebugUtilsMessengerEXT::null()
        };

        DebugMessenger { handle, device }
    }
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                self.device
                    .instance()
                    .destroy_debug_utils_messenger_ext(Some(self.handle), None);
            }
        }
    }
}

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL_EXT => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE_EXT => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION_EXT => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagBitsEXT::VERBOSE_EXT => {
            log::trace!("{} {:?}", types, message)
        }
        vk::DebugUtilsMessageSeverityFlagBitsEXT::INFO_EXT => {
            log::info!("{} {:?}", types, message)
        }
        vk::DebugUtilsMessageSeverityFlagBitsEXT::WARNING_EXT => {
            log::warn!("{} {:?}", types, message)
        }
        vk::DebugUtilsMessageSeverityFlagBitsEXT::ERROR_EXT => {
            log::error!("{} {:?}", types, message)
        }
        _ => log::warn!("{} {:?}", types, message),
    };

    vk::FALSE
}
