use crate::vulkan::instance::Instance;
use erupt::vk;
use winit::window::Window;

#[derive(Copy, Clone)]
pub struct Surface {
    handle: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(instance: &Instance, window: &Window) -> Self {
        let surface = unsafe {
            erupt::utils::surface::create_surface(instance.handle(), &window, None).unwrap()
        };

        Surface { handle: surface }
    }

    pub fn destroy(&self, instance: &Instance) {
        unsafe {
            instance.handle().destroy_surface_khr(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }
}
