use crate::app::App;
use erupt::vk;
use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod app;
mod triangle_pass;
mod vulkan;

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    log::info!("Starting RDX");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut app = App::new(&window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                app.resize(vk::Extent2D {
                    width: size.width,
                    height: size.height,
                });
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                app.draw();
            }
            _ => (),
        }
    });
}
