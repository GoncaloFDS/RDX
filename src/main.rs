use engine::RdxEngine;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

mod debug;
mod device;
mod engine;
mod renderer;
mod swapchain;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const WINDOW_NAME: &str = "Rdx - Vulkan";

fn main() {
    env_logger::init();

    let (mut engine, event_loop) = RdxEngine::new();

    engine.init_systems();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                log::debug!("Closing window");
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(PhysicalSize { width, height }),
                ..
            } => {
                log::info!("Resizing window: {}x{}", width, height);
                engine.resize();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(code),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if code == VirtualKeyCode::Escape {
                    *control_flow = ControlFlow::Exit
                }
            }
            Event::MainEventsCleared => {
                engine.run();
            }
            _ => (),
        }
    });
}
