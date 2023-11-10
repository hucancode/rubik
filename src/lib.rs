mod geometry;
mod shader;
mod world;

use geometry::Geometry;
use glam::Vec4;
use shader::Shader;
use std::f32::consts::PI;
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use world::{new_entity, new_group, new_light, Node};
const CUBE_SIZE: f32 = 2.0;
const CUBE_MARGIN: f32 = 0.15;
const MAX_FPS: u64 = 60;
const TARGET_FRAME_TIME: Duration = Duration::from_millis(1000 / MAX_FPS);

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut renderer = world::Renderer::new(&window).await;
    let cube_mesh = Arc::new(Geometry::new_cube(0xcba6f7ff, &renderer.device));
    let rubik_mesh = Arc::new(Geometry::new_rubik_piece(&renderer.device));
    let shader = Arc::new(Shader::new(
        &renderer.device,
        include_str!("shader/shader.wgsl"),
    ));
    let d = CUBE_SIZE + CUBE_MARGIN;
    let mut rows = Vec::new();
    for z in -1..=1 {
        let mut row = new_group();
        for y in -1..=1 {
            for x in -1..=1 {
                let mut cube = new_entity(cube_mesh.clone(), shader.clone());
                row.add_child(cube.clone());
                cube.translate(d * x as f32, d * y as f32, d * z as f32);
            }
        }
        rows.push(row.clone());
        renderer.root.add_child(row);
    }
    let light_datas = vec![
        (
            wgpu::Color {
                r: 0.0,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
            10.0,
            0,
        ),
        (
            wgpu::Color {
                r: 0.0,
                g: 0.5,
                b: 1.0,
                a: 1.0,
            },
            10.0,
            2200,
        ),
        (
            wgpu::Color {
                r: 0.0,
                g: 1.0,
                b: 0.5,
                a: 1.0,
            },
            10.0,
            4400,
        ),
    ];
    let mut lights = Vec::new();
    for (color, radius, time_offset) in light_datas {
        let mut cube = new_entity(rubik_mesh.clone(), shader.clone());
        cube.scale_uniform(0.5);
        cube.translate(1.0, 1.0, 1.0);
        let mut light = new_light(color, radius);
        light.add_child(cube.clone());
        renderer.root.add_child(light.clone());
        lights.push((light, cube, time_offset));
    }
    let app_start_time = Instant::now();
    let mut update = move || {
        let time = app_start_time.elapsed().as_millis();
        for (light, cube, time_offset) in lights.iter_mut() {
            let time = time + *time_offset as u128;
            let rx = PI * 2.0 * ((time as f64) * 0.00042).sin() as f32;
            let ry = PI * 2.0 * ((time as f64) * 0.00011).sin() as f32;
            let rz = PI * 2.0 * ((time as f64) * 0.00027).sin() as f32;
            cube.rotate(rx, ry, rz);
            let x = 4.0 * (time as f64 / 1700.0).sin() as f32;
            let y = 4.0 * (time as f64 / 1300.0).sin() as f32;
            let z = 4.0 * (time as f64 / 700.0).sin() as f32;
            let v = Vec4::new(x, y, z, 1.0).normalize() * 10.0;
            light.translate(v.x, v.y, v.z);
        }
        for (i, row) in rows.iter_mut().enumerate() {
            let alpha = PI * (1.0 + ((time as f64) * 0.0007 + (i as f64) * 0.08).sin() as f32);
            row.rotate_z(alpha);
        }
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
