use erupt::{vk, InstanceLoader};
use winit::window::Window;

#[derive(Copy, Clone)]
pub struct Surface {
    handle: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(instance: &InstanceLoader, window: &Window) -> Self {
        let surface =
            unsafe { erupt::utils::surface::create_surface(instance, &window, None).unwrap() };

        Surface { handle: surface }
    }

    pub fn destroy(&self, instance: &InstanceLoader) {
        unsafe {
            instance.destroy_surface_khr(self.handle, None);
        }
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }
}
