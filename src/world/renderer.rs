use crate::geometry::Vertex;
use crate::shader::Shader;
use crate::world::Camera;
use crate::world::Node;
use glam::Mat4;
use std::collections::VecDeque;
use wgpu::util::DeviceExt;
use wgpu::{
    util::BufferInitDescriptor, BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, Device, PipelineLayoutDescriptor, Queue, RenderPipeline,
    RenderPipelineDescriptor, Surface, SurfaceConfiguration,
};
use winit::window::Window;

pub struct Renderer {
    pub camera: Camera,
    pub root: Node,
    config: SurfaceConfiguration,
    surface: Surface,
    pub device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    bind_group_camera: BindGroup,
    bind_group_node: BindGroup,
    vp_buffer: Buffer,
    w_buffer: Buffer,
}

impl Renderer {
    pub async fn new(window: &Window) -> Renderer {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let bind_group_layout_camera =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0, // view projection
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                }],
            });
        let bind_group_layout_node = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0, // world
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout_node, &bind_group_layout_camera],
            push_constant_ranges: &[],
        });

        let shader = Shader::new(&device, include_str!("../shader/shader.wgsl"));

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vp = Camera::make_vp_matrix(config.width as f32 / config.height as f32);
        let vp_ref: &[f32; 16] = vp.as_ref();
        let vp_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera View Projection Buffer"),
            contents: bytemuck::cast_slice(vp_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_camera = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_camera,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: vp_buffer.as_entire_binding(),
            }],
            label: None,
        });
        let w_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(Mat4::IDENTITY.as_ref()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_node = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_node,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: w_buffer.as_entire_binding(),
            }],
            label: None,
        });
        Self {
            camera: Camera::new(),
            root: Node::new_empty(),
            config,
            surface,
            device,
            queue,
            render_pipeline,
            bind_group_node,
            bind_group_camera,
            vp_buffer,
            w_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        let mvp = Camera::make_vp_matrix(self.config.width as f32 / self.config.height as f32);
        let mvp_ref: &[f32; 16] = mvp.as_ref();
        self.queue
            .write_buffer(&self.vp_buffer, 0, bytemuck::cast_slice(mvp_ref));
    }

    pub fn draw(&self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(0, &self.bind_group_camera, &[]);
            rpass.set_bind_group(1, &self.bind_group_node, &[]);
            let mut q = Vec::new();
            q.push(&self.root);
            while let Some(node) = q.pop() {
                if let Some(visual) = &node.visual {
                    self.queue.write_buffer(
                        &self.w_buffer,
                        0,
                        bytemuck::cast_slice(node.transform.lock().unwrap().calculate().as_ref()),
                    );
                    rpass.set_index_buffer(
                        visual.geometry.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    rpass.set_vertex_buffer(0, visual.geometry.vertex_buffer.slice(..));
                    let n = visual.geometry.indices.len() as u32;
                    rpass.draw_indexed(0..n, 0, 0..1);
                }
                for child in node.children.iter() {
                    q.push(child);
                }
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
