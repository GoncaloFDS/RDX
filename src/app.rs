use crate::renderers::clear::Clear;
use crate::renderers::egui_renderer::EguiRenderer;
use crate::renderers::model_renderer::ModelRenderer;
use crate::renderers::Renderer;
use crate::user_interface::UserInterface;
use crate::vulkan::device::Device;
use crate::vulkan::instance::Instance;
use crate::vulkan::subresource_range::SubresourceRange;
use erupt::vk;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct App {
    window: Window,
    device: Device,
    instance: Instance,
    clear: Clear,
    model_renderer: ModelRenderer,
    egui_renderer: EguiRenderer,
    ui: UserInterface,
}

impl App {
    pub fn new() -> (Self, EventLoop<()>) {
        log::info!("Starting RDX");
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let instance = Instance::new(&window);
        let mut device = Device::new(&instance, &window);

        let ui = UserInterface::new(&window);

        let clear = Clear::new(&device);
        let model_renderer = ModelRenderer::new(&device, device.surface_format());
        let egui_renderer = EguiRenderer::new(&mut device, &ui);

        let app = App {
            window,
            device,
            instance,
            clear,
            model_renderer,
            egui_renderer,
            ui,
        };

        (app, event_loop)
    }

    pub fn on_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event, .. } => {
                let ui_captured_input = self.ui.on_event(&event);
                if ui_captured_input {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Closing Window");
                        *control_flow = ControlFlow::Exit
                    }
                    WindowEvent::Resized(size) => self.resize(vk::Extent2D {
                        width: size.width,
                        height: size.height,
                    }),
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                self.draw();
            }
            _ => (),
        }
    }

    pub fn resize(&mut self, size: vk::Extent2D) {
        self.device.resize_swapchain(size);
        self.clear = Clear::new(&self.device);
    }

    pub fn draw(&mut self) {
        self.ui.update(&self.window);
        self.egui_renderer
            .update_buffers(&self.device, &mut self.ui);
        self.egui_renderer
            .update_textures(&mut self.device, &self.ui);

        let acquired_frame = self
            .device
            .acquire_swapchain_frame(&self.instance, u64::MAX);

        if acquired_frame.invalidate_images {
            self.recreate_swapchain();
        }

        let command_buffer = self.device.command_buffer(acquired_frame.frame_index);
        let semaphore = self.device.semaphore(acquired_frame.frame_index);

        command_buffer.begin(&self.device, vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

        let swapchain_image = self.device.swapchain_image(acquired_frame.image_index);

        command_buffer.image_memory_barrier(
            &self.device,
            swapchain_image,
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::NONE,
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );

        self.clear
            .fill_command_buffer(&self.device, command_buffer, acquired_frame.image_index);

        self.model_renderer.fill_command_buffer(
            &self.device,
            command_buffer,
            acquired_frame.image_index,
        );

        self.egui_renderer.fill_command_buffer(
            &self.device,
            command_buffer,
            acquired_frame.image_index,
        );

        command_buffer.end_rendering(&self.device);

        command_buffer.image_memory_barrier(
            &self.device,
            swapchain_image,
            SubresourceRange::with_aspect(vk::ImageAspectFlags::COLOR),
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        command_buffer.end(&self.device);

        self.device.submit(
            &[vk::SemaphoreSubmitInfoBuilder::new()
                .semaphore(acquired_frame.ready)
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)],
            &[vk::SemaphoreSubmitInfoBuilder::new()
                .semaphore(semaphore.handle())
                .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)],
            &[vk::CommandBufferSubmitInfoBuilder::new().command_buffer(command_buffer.handle())],
            acquired_frame.complete,
        );

        self.device.queue_present(
            self.device.queue(),
            semaphore.handle(),
            acquired_frame.image_index,
        );
    }

    fn recreate_swapchain(&mut self) {
        self.device.recreate_swapchain();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device.destroy();
        self.instance.destroy();
    }
}
