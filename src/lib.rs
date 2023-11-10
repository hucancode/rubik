mod geometry;
mod material;
mod world;
mod app;
use crate::app::App;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut app = App::new(&window).await;
    event_loop.run(move |event, _, control_flow| {
        app.update();
        window.request_redraw();
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                app.resize(size.width, size.height);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                app.draw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
