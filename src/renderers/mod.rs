use crate::user_interface::UserInterface;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;

pub mod clear;
pub mod egui_renderer;
pub mod model_renderer;

pub trait Renderer {
    fn fill_command_buffer(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
        current_image: usize,
    );

    fn update(&mut self, device: &mut Device, ui: &mut UserInterface);

    fn destroy(&mut self, device: &mut Device);
}
