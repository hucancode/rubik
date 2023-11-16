mod app;
mod geometry;
mod material;
mod rubik;
mod world;
use crate::app::App;
use std::time::Instant;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut app = App::new(&window).await;
    let mut last_update_timestamp = Instant::now();
    let app_start_timestamp = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::NewEvents(StartCause::Init) => {
                app.init();
            }
            Event::NewEvents(StartCause::Poll) => {
                let delta_time = 0.001 * last_update_timestamp.elapsed().as_millis() as f32;
                let time = app_start_timestamp.elapsed().as_millis();
                app.update(delta_time, time);
                last_update_timestamp = Instant::now();
                window.request_redraw();
            }
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
