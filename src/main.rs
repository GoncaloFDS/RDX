use crate::app::App;

mod app;
mod block;
mod camera;
mod chunk;
mod input;
mod model;
mod renderers;
mod scene;
mod time;
mod user_interface;
mod vulkan;

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let (mut app, event_loop) = App::new();

    event_loop.run(move |event, _, control_flow| {
        app.on_event(event, control_flow);
    });
}
