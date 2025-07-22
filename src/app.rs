use crate::geometry::Mesh;
use crate::material::ShaderUnlit;
use crate::rubik::{Rubik, Move};
use crate::world::{new_entity, new_light, Node, NodeRef, Renderer};
use egui_winit::State as EguiState;
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
use winit::event::{ElementState, MouseButton, StartCause, WindowEvent};
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
    egui_state: Option<EguiState>,
    egui_ctx: egui::Context,
    mouse_down: bool,
    last_mouse_pos: (f32, f32),
    egui_frame_started: bool,
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
            egui_state: None,
            egui_ctx: {
                let ctx = egui::Context::default();
                let fonts = egui::FontDefinitions::default();
                // Ensure we have fonts at various sizes
                ctx.set_fonts(fonts);
                ctx.set_pixels_per_point(1.0); // Default scale, will be updated later
                ctx
            },
            mouse_down: false,
            last_mouse_pos: (0.0, 0.0),
            egui_frame_started: false,
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
        
        // Update egui
        if let Some(egui_state) = self.egui_state.as_mut() {
            if let Some(window) = self.window.as_ref() {
                let raw_input = egui_state.take_egui_input(window);
                self.egui_ctx.begin_pass(raw_input);
                self.egui_frame_started = true;
                
                // Create debug GUI
                egui::Window::new("Debug Controls")
                    .show(&self.egui_ctx, |ui| {
                        ui.heading("Rubik's Cube Controls");
                        
                        ui.separator();
                        
                        if ui.button(if self.rubik.paused { "Resume" } else { "Pause" }).clicked() {
                            self.rubik.paused = !self.rubik.paused;
                        }
                        
                        ui.checkbox(&mut self.rubik.auto_move, "Auto Move");
                        
                        ui.separator();
                        ui.label("Manual Rotation:");
                        
                        ui.horizontal(|ui| {
                            if ui.button("U").clicked() {
                                self.rubik.perform_move(Move::Top);
                            }
                            if ui.button("D").clicked() {
                                self.rubik.perform_move(Move::Bottom);
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            if ui.button("L").clicked() {
                                self.rubik.perform_move(Move::Left);
                            }
                            if ui.button("R").clicked() {
                                self.rubik.perform_move(Move::Right);
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            if ui.button("F").clicked() {
                                self.rubik.perform_move(Move::Front);
                            }
                            if ui.button("B").clicked() {
                                self.rubik.perform_move(Move::Back);
                            }
                        });
                        
                        ui.separator();
                        ui.label("Camera Controls:");
                        ui.label("• Left mouse drag: Orbit");
                        ui.label("• Mouse wheel: Zoom");
                        
                        ui.separator();
                        let camera = &renderer.camera;
                        ui.label(format!("Distance: {:.1}", camera.distance));
                        ui.label(format!("Azimuth: {:.2}", camera.azimuth));
                        ui.label(format!("Elevation: {:.2}", camera.elevation));
                    });
            }
        }
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
            attr = attr.with_title("Rubik");
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
        
        // Initialize egui state
        if let Some(window) = self.window.as_ref() {
            // Create a new context for egui
            let egui_ctx = egui::Context::default();
            
            // Set up fonts before creating state
            egui_ctx.set_fonts(egui::FontDefinitions::default());
            
            // Get scale factor from window
            let scale_factor = window.scale_factor() as f32;
            egui_ctx.set_pixels_per_point(scale_factor);
            
            let viewport_id = egui_ctx.viewport_id();
            let egui_state = EguiState::new(
                egui_ctx.clone(),
                viewport_id,
                window,
                None,
                None,
                None,
            );
            self.egui_ctx = egui_ctx;
            self.egui_state = Some(egui_state);
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Pass event to egui first
        let mut egui_consumed = false;
        if let Some(egui_state) = self.egui_state.as_mut() {
            if let Some(window) = self.window.as_ref() {
                let response = egui_state.on_window_event(window, &event);
                egui_consumed = response.consumed;
            }
        }
        
        if event == WindowEvent::CloseRequested {
            event_loop.exit();
        } else if let Some(renderer) = self.renderer.as_mut() {
            match event {
                WindowEvent::RedrawRequested => {
                    if let Some(egui_state) = self.egui_state.as_mut() {
                        if let Some(window) = self.window.as_ref() {
                            if self.egui_frame_started {
                                let egui_output = self.egui_ctx.end_pass();
                                egui_state.handle_platform_output(window, egui_output.platform_output);
                                
                                let clipped_primitives = self.egui_ctx.tessellate(
                                    egui_output.shapes,
                                    self.egui_ctx.pixels_per_point(),
                                );
                                
                                renderer.draw(&self.egui_ctx, clipped_primitives, egui_output.textures_delta);
                                self.egui_frame_started = false;
                            } else {
                                // No egui frame active, render without egui
                                let empty_primitives = Vec::new();
                                let empty_textures = egui::TexturesDelta::default();
                                let dummy_ctx = egui::Context::default();
                                dummy_ctx.set_fonts(egui::FontDefinitions::default());
                                dummy_ctx.set_pixels_per_point(1.0);
                                renderer.draw(&dummy_ctx, empty_primitives, empty_textures);
                            }
                        }
                    } else {
                        // Don't render egui until it's properly initialized
                        let empty_primitives = Vec::new();
                        let empty_textures = egui::TexturesDelta::default();
                        // Create a dummy context that won't be used
                        let dummy_ctx = egui::Context::default();
                        dummy_ctx.set_fonts(egui::FontDefinitions::default());
                        dummy_ctx.set_pixels_per_point(1.0);
                        renderer.draw(&dummy_ctx, empty_primitives, empty_textures);
                    }
                },
                WindowEvent::Resized(size) => renderer.resize(size.width, size.height),
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    self.egui_ctx.set_pixels_per_point(scale_factor as f32);
                }
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
                WindowEvent::MouseInput { state, button, .. } => {
                    if !egui_consumed && button == MouseButton::Left {
                        self.mouse_down = state == ElementState::Pressed;
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let current_pos = (position.x as f32, position.y as f32);
                    
                    if !egui_consumed && self.mouse_down {
                        let delta_x = current_pos.0 - self.last_mouse_pos.0;
                        let delta_y = current_pos.1 - self.last_mouse_pos.1;
                        
                        renderer.camera.orbit(-delta_x * 0.01, delta_y * 0.01);
                    }
                    
                    self.last_mouse_pos = current_pos;
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    if !egui_consumed {
                        use winit::event::MouseScrollDelta;
                        let zoom_delta = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y * 2.0,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.05,
                        };
                        renderer.camera.zoom(-zoom_delta);
                    }
                }
                _ => {}
            }
        }
    }
}
