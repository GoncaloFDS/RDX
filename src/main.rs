use engine::Engine;
use simple_logger::SimpleLogger;

mod block;
mod camera;
mod chunk;
mod engine;
mod input;
mod time;
mod user_interface;
mod vulkan;

const WIDTH: u32 = 1400;
const HEIGHT: u32 = 900;
const WINDOW_NAME: &str = "Rdx - Vulkan";

fn main() {
    SimpleLogger::new().init().unwrap();

    let (mut engine, event_loop) = Engine::new(WIDTH, HEIGHT, WINDOW_NAME);

    event_loop.run(move |event, _, control_flow| {
        engine.on_event(event, control_flow);
    });
}
