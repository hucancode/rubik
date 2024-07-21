use crate::geometry::Mesh;
use crate::material::ShaderUnlit;
use crate::rubik::Rubik;
use crate::world::{new_entity, new_light, Node, NodeRef, Renderer};
use glam::Vec4;
use std::f32::consts::PI;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use wgpu::Color;
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

const LIGHT_RADIUS: f32 = 10.0;
const LIGHT_INTENSITY: f32 = 2.5;

pub struct App {
    window: Option<Arc<Window>>,
    start_time_stamp: Instant,
    last_frame_timestamp: Instant,
    renderer: Option<Renderer>,
    lights: Vec<(NodeRef, NodeRef, u128)>,
    rubik: Rubik,
}

impl Default for App {
    fn default() -> Self {
        Self {
            window: None,
            start_time_stamp: Instant::now(),
            last_frame_timestamp: Instant::now(),
            renderer: None,
            lights: Vec::new(),
            rubik: Rubik::new(),
        }
    }
}

impl App {
    pub async fn init(&mut self) {
        if self.window.is_none() {
            return;
        }
        let mut renderer = Renderer::new(self.window.as_ref().unwrap().clone()).await;
        let app_init_timestamp = Instant::now();
        let cube_mesh = Rc::new(Mesh::new_cube(0xcba6f7ff, &renderer.device));
        let shader_unlit = Rc::new(ShaderUnlit::new(&renderer));
        self.rubik.generate_pieces(1, &renderer);
        self.rubik.start_move_random();
        renderer.root.add_child(self.rubik.root.clone());
        let lights = vec![
            (
                Color {
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
                Color {
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
                Color {
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
                renderer.root.add_child(light.clone());
                (light, cube, time_offset)
            })
            .collect();
        println!("app initialized in {:?}", app_init_timestamp.elapsed());
        self.renderer = Some(renderer);
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
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.time = time as f32;
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("Rubik"))
            .unwrap();
        self.window = Some(Arc::new(window));
        pollster::block_on(self.init());
    }
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Poll {
            let time = self.start_time_stamp.elapsed().as_millis();
            let delta_time = self.last_frame_timestamp.elapsed().as_secs_f32();
            self.update(delta_time, time);
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            self.last_frame_timestamp = Instant::now();
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if event == WindowEvent::CloseRequested {
            event_loop.exit();
        } else if let Some(renderer) = self.renderer.as_mut() {
            match event {
                WindowEvent::RedrawRequested => renderer.draw(),
                WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
                _ => {}
            }
        }
    }
}
