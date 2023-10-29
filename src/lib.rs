mod geometry;
mod shader;
mod world;

use geometry::Geometry;
use shader::Shader;
use std::time::Instant;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use world::Node;

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut renderer = world::Renderer::new(&window).await;
    let cube_mesh = Geometry::new_rubik_piece(&renderer.device);
    let cube_shader = Shader::new(&renderer.device, include_str!("shader/shader.wgsl"));
    let cube = Node::new(cube_mesh, cube_shader);
    let transform = cube.transform.clone();
    renderer.root.add_child(cube);
    const ROTATION_CYCLE: f32 = 2.0 * 3.14;
    const ROTATION_SPEED: f32 = 0.5 * 3.14;
    let mut rotation = 0.0;
    let mut last_update_time = Instant::now();
    let mut update = move || {
        let delta_time = last_update_time.elapsed().as_millis();
        rotation += ROTATION_SPEED * (delta_time as f64 / 1000.0) as f32;
        if rotation >= ROTATION_CYCLE {
            rotation -= ROTATION_CYCLE;
        }
        println!("rotation = {}", rotation);
        transform.lock().unwrap().rotation = glam::Quat::from_rotation_y(rotation);
        last_update_time = Instant::now();
    };
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        update();
        window.request_redraw();
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                renderer.resize(size.width, size.height);
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                renderer.draw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
