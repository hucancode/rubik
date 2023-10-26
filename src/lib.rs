mod shader;
mod geometry;
mod world;

use geometry::Geometry;
use shader::Shader;
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
    renderer.root.add_child(cube);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
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
