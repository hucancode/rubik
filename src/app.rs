use crate::geometry::Mesh;
use crate::material::ShaderUnlit;
use crate::rubik::Rubik;
use crate::world::{new_entity, new_light, Node, NodeRef, Renderer};
use glam::Vec4;
use std::f32::consts::PI;
use std::rc::Rc;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
use wgpu::Color;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const LIGHT_RADIUS: f32 = 10.0;
const LIGHT_INTENSITY: f32 = 2.5;
const WINDOW_WIDTH: u32 = 1024;
const WINDOW_HEIGHT: u32 = 768;

pub struct App {
    window: Option<Arc<Window>>,
    start_time_stamp: Instant,
    last_frame_timestamp: Instant,
    renderer: Option<Renderer>,
    lights: Vec<(NodeRef, NodeRef, u128)>,
    event_loop: Option<EventLoopProxy<Renderer>>,
    rubik: Rubik,
}

impl App {
    pub fn new(event_loop: &EventLoop<Renderer>) -> Self {
        Self {
            window: None,
            start_time_stamp: Instant::now(),
            last_frame_timestamp: Instant::now(),
            renderer: None,
            lights: Vec::new(),
            event_loop: Some(event_loop.create_proxy()),
            rubik: Rubik::new(),
        }
    }
}

impl App {
    pub async fn make_renderer(window: Arc<Window>) -> Renderer {
        Renderer::new(window.clone(), WINDOW_WIDTH, WINDOW_HEIGHT).await
    }
    pub fn init(&mut self) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
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
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        renderer.time = time as f32;
    }
}

impl ApplicationHandler<Renderer> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        use winit::dpi::PhysicalSize;
        log::info!("creating window...");
        let mut attr = Window::default_attributes()
            .with_inner_size(PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use web_sys::HtmlCanvasElement;
            use wgpu::web_sys;
            use winit::platform::web::WindowAttributesExtWebSys;
            // use first canvas element, or create one if none found
            let canvas = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector("canvas").ok())
                .and_then(|c| c)
                .and_then(|c| c.dyn_into::<HtmlCanvasElement>().ok());
            if let Some(canvas) = canvas {
                attr = attr.with_canvas(Some(canvas));
            } else {
                attr = attr.with_append(true);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            attr = attr.with_title("Dragon");
        }
        let window = Arc::new(event_loop.create_window(attr).unwrap());
        let Some(event_loop) = self.event_loop.take() else {
            return;
        };
        self.window = Some(window.clone());
        log::info!(
            "window created! inner size {:?} outer size {:?}",
            window.inner_size(),
            window.outer_size(),
        );
        log::info!("creating renderer...");
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                let renderer = App::make_renderer(window).await;
                log::info!("renderer created!");
                if let Err(_renderer) = event_loop.send_event(renderer) {
                    log::error!("Failed to send renderer back to application thread");
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let renderer = pollster::block_on(App::make_renderer(window));
            if let Err(_renderer) = event_loop.send_event(renderer) {
                log::error!("Failed to send renderer back to application thread");
            }
        }
    }
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Poll {
            let time = self.start_time_stamp.elapsed().as_millis();
            let delta_time = self.last_frame_timestamp.elapsed().as_secs_f32();
            self.update(delta_time, time);
            let Some(window) = self.window.as_ref() else {
                return;
            };
            window.request_redraw();
            self.last_frame_timestamp = Instant::now();
        }
    }
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, renderer: Renderer) {
        log::info!("got renderer!");
        self.renderer = Some(renderer);
        self.init();
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
                WindowEvent::KeyboardInput {
                    device_id: _dev,
                    event,
                    is_synthetic: _synthetic,
                } => {
                    log::info!("keyboard pressed {:?}", event);
                    match (event.physical_key, event.state) {
                        // space to restart animation
                        (PhysicalKey::Code(KeyCode::Space), ElementState::Released) => {
                            self.start_time_stamp = Instant::now();
                        }
                        // escape to exit
                        (PhysicalKey::Code(KeyCode::Escape), ElementState::Released) => {
                            event_loop.exit();
                        }
                        // P to pause/play animation
                        (PhysicalKey::Code(KeyCode::KeyP), ElementState::Released) => {
                            match event_loop.control_flow() {
                                ControlFlow::Poll => event_loop.set_control_flow(ControlFlow::Wait),
                                ControlFlow::Wait => event_loop.set_control_flow(ControlFlow::Poll),
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
