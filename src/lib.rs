mod geometry;
mod shader;
mod world;

use geometry::Geometry;
use shader::Shader;
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use world::Node;

const ROTATE_CYCLE: u128 = 5000;
const MAX_FPS: u64 = 60;
const TARGET_FRAME_TIME: Duration = Duration::from_millis(1000 / MAX_FPS);

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut renderer = world::Renderer::new(&window).await;
    let cube_mesh = Arc::new(Geometry::new_cube(0x0077ff, &renderer.device));
    let rubik_mesh = Arc::new(Geometry::new_rubik_piece(&renderer.device));
    let shader = Arc::new(Shader::new(
        &renderer.device,
        include_str!("shader/shader.wgsl"),
    ));
    let rubik = Node::new(rubik_mesh, shader.clone());
    let cube = Node::new(cube_mesh, shader);
    let transform = rubik.transform.clone();
    renderer.root.add_child(Arc::new(rubik));
    renderer.root.add_child(Arc::new(cube));
    let app_start_time = Instant::now();
    let update = move || {
        let time = app_start_time.elapsed().as_millis();
        let rotation = 2.0 * 3.14 * 0.001 * (time % ROTATE_CYCLE) as f32;
        let mut transform = transform.lock().unwrap();
        transform.rotation = glam::Quat::from_rotation_z(rotation);
        let z = (time as f64 / 1000.0).sin() as f32;
        transform.translation = glam::Vec3::new(0.0, 0.0, z);
    };
    let mut last_update_time = Instant::now();
    event_loop.run(move |event, _, control_flow| {
        // println!(
        //     "FPS: {}",
        //     1000.0 / last_update_time.elapsed().as_millis() as f64
        // );
        update();
        window.request_redraw();
        *control_flow = ControlFlow::WaitUntil(last_update_time.add(TARGET_FRAME_TIME));
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
        last_update_time = Instant::now();
    });
}
