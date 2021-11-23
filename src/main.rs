use engine::Engine;
use simple_logger::SimpleLogger;
use winit::{
    event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

mod camera;
mod engine;
mod input;
mod time;
mod vulkan;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const WINDOW_NAME: &str = "Rdx - Vulkan";

fn main() {
    SimpleLogger::new().init().unwrap();

    let (mut engine, event_loop) = Engine::new(WIDTH, HEIGHT, WINDOW_NAME);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => match window_event {
                WindowEvent::Resized(size) => {
                    log::info!("Resizing window: {}x{}", size.width, size.height);
                    engine.resize();
                }
                WindowEvent::CloseRequested => {
                    log::debug!("Closing window");
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                        engine.shutdown();
                        *control_flow = ControlFlow::Exit
                    } else {
                        engine.handle_key_input(input)
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    engine.handle_mouse_move(position);
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    engine.handle_mouse_input(button, state);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                engine.run();
            }
            _ => (),
        }
    });
}
