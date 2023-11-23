use crate::geometry::Mesh;
use crate::material::ShaderUnlit;
use crate::rubik::Rubik;
use crate::world::{new_entity, new_light, Node, NodeRef, Renderer};
use glam::Vec4;
use std::f32::consts::PI;
use std::rc::Rc;
use std::time::Instant;
use winit::window::Window;

const LIGHT_RADIUS: f32 = 10.0;
const LIGHT_INTENSITY: f32 = 2.5;

pub struct App {
    renderer: Renderer,
    lights: Vec<(NodeRef, NodeRef, u128)>,
    rubik: Rubik,
}

impl App {
    pub async fn new(window: &Window) -> Self {
        let renderer = Renderer::new(window).await;
        Self {
            renderer,
            lights: Vec::new(),
            rubik: Rubik::new(),
        }
    }
    pub fn init(&mut self) {
        let app_init_timestamp = Instant::now();
        let cube_mesh = Rc::new(Mesh::new_cube(0xcba6f7ff, &self.renderer.device));
        let shader_unlit = Rc::new(ShaderUnlit::new(&self.renderer));
        self.rubik.generate_pieces(1, &self.renderer);
        self.rubik.start_move_random();
        self.renderer.root.add_child(self.rubik.root.clone());
        let lights = vec![
            (
                wgpu::Color {
                    r: 0.0,
                    g: 0.5,
                    b: 1.0,
                    a: 1.0,
                },
                LIGHT_RADIUS,
                LIGHT_INTENSITY,
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
                LIGHT_INTENSITY,
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
                LIGHT_INTENSITY,
                4400,
            ),
        ];
        self.lights = lights
            .into_iter()
            .map(|(color, radius, intensity, time_offset)| {
                let mut cube = new_entity(cube_mesh.clone(), shader_unlit.clone());
                cube.scale_uniform(0.7);
                cube.translate(1.0, 1.0, 1.0);
                let mut light = new_light(color, radius * intensity);
                light.add_child(cube.clone());
                self.renderer.root.add_child(light.clone());
                (light, cube, time_offset)
            })
            .collect();
        println!("app initialized in {:?}", app_init_timestamp.elapsed());
    }
    pub fn update(&mut self, delta_time: f32, time: u128) {
        for (light, cube, time_offset) in self.lights.iter_mut() {
            let time = time + *time_offset;
            let rx = PI * 2.0 * (0.00042 * time as f64).sin() as f32;
            let ry = PI * 2.0 * (0.00011 * time as f64).sin() as f32;
            let rz = PI * 2.0 * (0.00027 * time as f64).sin() as f32;
            cube.rotate(rx, ry, rz);
            let x = 4.0 * (0.00058 * time as f64).sin() as f32;
            let y = 4.0 * (0.00076 * time as f64).sin() as f32;
            let z = 4.0 * (0.00142 * time as f64).sin() as f32;
            let v = Vec4::new(x, y, z, 1.0).normalize() * LIGHT_RADIUS;
            light.translate(v.x, v.y, v.z);
        }
        self.rubik.update(delta_time);
        self.rubik.root.rotate_z((0.0003 * time as f64) as f32);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn draw(&self) {
        self.renderer.draw();
    }
}
