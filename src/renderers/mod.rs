use crate::camera::Camera;
use crate::scene::Scene;
use crate::user_interface::UserInterface;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::device::Device;
use bevy_ecs::prelude::*;
use downcast_rs::{impl_downcast, Downcast};

pub mod clear;
pub mod egui_renderer;
pub mod model_renderer;
pub mod raytracer;

pub trait Renderer: Downcast {
    fn fill_command_buffer(
        &self,
        device: &Device,
        command_buffer: &CommandBuffer,
        current_image: usize,
    );

    fn update(
        &mut self,
        device: &mut Device,
        current_image: usize,
        camera: &Camera,
        ui: &mut UserInterface,
        world: &mut World,
        scene: &Scene,
    );

    fn destroy(&mut self, device: &mut Device);
}
impl_downcast!(Renderer);
