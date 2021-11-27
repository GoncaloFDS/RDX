use crate::vulkan::device::Device;
use erupt::utils::surface;
use erupt::vk;
use std::rc::Rc;
use winit::window::Window;

pub struct Surface {
    handle: vk::SurfaceKHR,
    device: Rc<Device>,
}

impl Surface {
    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }

    pub fn uninitialized(device: Rc<Device>) -> Self {
        Surface {
            handle: Default::default(),
            device,
        }
    }

    pub fn new(device: Rc<Device>, window: &Window) -> Self {
        let surface = unsafe { surface::create_surface(device.instance(), window, None).unwrap() };

        Surface {
            handle: surface,
            device,
        }
    }

    pub fn cleanup(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                self.device
                    .instance()
                    .destroy_surface_khr(Some(self.handle), None)
            }
        }
    }
}
