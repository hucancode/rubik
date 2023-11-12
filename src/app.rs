use crate::geometry::Mesh;
use crate::material::Shader;
use crate::world::{new_entity, new_group, new_light, Node, NodeRef, Renderer};
use glam::Vec4;
use std::f32::consts::PI;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::window::Window;

const CUBE_SIZE: f32 = 2.0;
const CUBE_MARGIN: f32 = 0.15;
const MAX_FPS: u64 = 60;
const TARGET_FRAME_TIME: Duration = Duration::from_millis(1000 / MAX_FPS);
const LIGHT_RADIUS: f32 = 30.0;

pub struct App {
    renderer: Renderer,
    start_timestamp: Instant,
    last_update_timestamp: Instant,
    lights: Vec<(NodeRef, NodeRef, u128)>,
    rows: Vec<NodeRef>,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let mut renderer = Renderer::new(window).await;
        let cube_mesh = Arc::new(Mesh::new_cube(0xcba6f7ff, &renderer.device));
        let rubik_mesh = Arc::new(Mesh::new_rubik_piece(&renderer.device));
        let shader = Arc::new(Shader::new(
            &renderer.device,
            include_str!("material/shader.wgsl"),
        ));
        let d = CUBE_SIZE + CUBE_MARGIN;
        let mut rows = Vec::new();
        let n = 4;
        for z in -n..=n {
            let mut row = new_group();
            for y in -n..=n {
                for x in -n..=n {
                    let mut cube = new_entity(cube_mesh.clone(), shader.clone());
                    row.add_child(cube.clone());
                    cube.translate(d * x as f32, d * y as f32, d * z as f32);
                }
            }
            rows.push(row.clone());
            renderer.root.add_child(row);
        }
        let lights = vec![
            (
                wgpu::Color {
                    r: 0.0,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                },
                LIGHT_RADIUS,
                0,
            ),
            (
                wgpu::Color {
                    r: 0.0,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                },
                LIGHT_RADIUS,
                2200,
            ),
            (
                wgpu::Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.5,
                    a: 1.0,
                },
                LIGHT_RADIUS,
                4400,
            ),
        ];
        let lights = lights
            .into_iter()
            .map(|(color, radius, time_offset)| {
                let mut cube = new_entity(rubik_mesh.clone(), shader.clone());
                cube.scale_uniform(0.5);
                cube.translate(1.0, 1.0, 1.0);
                let mut light = new_light(color, radius);
                light.add_child(cube.clone());
                renderer.root.add_child(light.clone());
                (light, cube, time_offset)
            })
            .collect();
        Self {
            renderer,
            start_timestamp: Instant::now(),
            last_update_timestamp: Instant::now(),
            lights,
            rows,
        }
    }
    pub fn update(&mut self) {
        if self.last_update_timestamp.elapsed() < TARGET_FRAME_TIME {
            return;
        }
        let time = self.start_timestamp.elapsed().as_millis();
        for (light, cube, time_offset) in self.lights.iter_mut() {
            let time = time + *time_offset;
            let rx = PI * 2.0 * ((time as f64) * 0.00042).sin() as f32;
            let ry = PI * 2.0 * ((time as f64) * 0.00011).sin() as f32;
            let rz = PI * 2.0 * ((time as f64) * 0.00027).sin() as f32;
            cube.rotate(rx, ry, rz);
            let x = 4.0 * (time as f64 / 1700.0).sin() as f32;
            let y = 4.0 * (time as f64 / 1300.0).sin() as f32;
            let z = 4.0 * (time as f64 / 700.0).sin() as f32;
            let v = Vec4::new(x, y, z, 1.0).normalize() * LIGHT_RADIUS;
            light.translate(v.x, v.y, v.z);
        }
        for (i, row) in self.rows.iter_mut().enumerate() {
            let alpha = PI * (1.0 + ((time as f64) * 0.0007 + (i as f64) * 0.08).sin() as f32);
            row.rotate_z(alpha);
        }
        self.last_update_timestamp = Instant::now();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn draw(&self) {
        self.renderer.draw();
    }
}
