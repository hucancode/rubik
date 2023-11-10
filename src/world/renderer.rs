use crate::geometry::Vertex;
use crate::shader::Shader;
use crate::world::{node, Camera, Light, Node, NodeRef};
use glam::{Mat4, Vec4};
use std::cmp::max;
use std::mem::size_of;
use wgpu::util::{align_to, BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayoutDescriptor, BindGroupLayoutEntry, Buffer,
    BufferDescriptor, Device, PipelineLayoutDescriptor, Queue, RenderPipeline,
    RenderPipelineDescriptor, Surface, SurfaceConfiguration,
};
use winit::window::Window;

const MAX_ENTITY: u64 = 100000;
const MAX_LIGHT: u64 = 10;
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.06666666666,
    g: 0.06666666666,
    b: 0.10588235294,
    a: 1.0,
};

pub struct Renderer {
    pub camera: Camera,
    pub root: NodeRef,
    config: SurfaceConfiguration,
    surface: Surface,
    pub device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    depth_texture_view: wgpu::TextureView,
    bind_group_camera: BindGroup,
    bind_group_node: BindGroup,
    vp_buffer: Buffer,
    w_buffer: Buffer,
    light_buffer: Buffer,
    light_count_buffer: Buffer,
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
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, // view projection
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1, // light
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(0),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2, // light count
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(8),
                        },
                        count: None,
                    },
                ],
            });
        let bind_group_layout_node = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0, // world
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
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

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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

        let light_uniform_size = size_of::<Light>() as wgpu::BufferAddress;
        let light_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Light Buffer"),
            size: MAX_LIGHT as wgpu::BufferAddress * light_uniform_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_count_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Light Count"),
            size: 8,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_camera = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_camera,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: vp_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_count_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        let node_uniform_size = size_of::<Mat4>() as wgpu::BufferAddress;
        let node_uniform_aligned = {
            let alignment =
                device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
            align_to(node_uniform_size, alignment)
        };
        let w_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Model Buffer"),
            size: MAX_ENTITY as wgpu::BufferAddress * node_uniform_aligned,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_node = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_node,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &w_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(node_uniform_size),
                }),
            }],
            label: None,
        });
        Self {
            camera: Camera::new(),
            root: node::new_group(),
            config,
            surface,
            device,
            queue,
            render_pipeline,
            depth_texture_view,
            bind_group_node,
            bind_group_camera,
            vp_buffer,
            w_buffer,
            light_buffer,
            light_count_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = max(1, width);
        self.config.height = max(1, height);
        self.surface.configure(&self.device, &self.config);
        let mvp = Camera::make_vp_matrix(self.config.width as f32 / self.config.height as f32);
        let mvp_ref: &[f32; 16] = mvp.as_ref();
        self.queue
            .write_buffer(&self.vp_buffer, 0, bytemuck::cast_slice(mvp_ref));
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        self.depth_texture_view =
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
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
        let mut nodes = Vec::new();
        let mut lights: Vec<(wgpu::Color, f32, Mat4)> = Vec::new();
        {
            nodes.clear();
            lights.clear();
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.set_bind_group(1, &self.bind_group_camera, &[]);
            let mut q = Vec::new();
            q.push((self.root.clone(), Mat4::IDENTITY));
            while let Some((node, transform_mx)) = q.pop() {
                let node = node.lock().unwrap();
                match &node.variant {
                    node::Variant::Entity(geometry, shader) => {
                        nodes.push((geometry.clone(), shader.clone(), transform_mx));
                    }
                    node::Variant::Light(color, radius) => {
                        lights.push((*color, *radius, transform_mx));
                    }
                    _ => {}
                }
                for child in node.children.iter() {
                    let transform_mx = transform_mx * child.calculate_transform();
                    q.push((child.clone(), transform_mx));
                }
            }
            self.queue.write_buffer(
                &self.light_count_buffer,
                0,
                bytemuck::bytes_of(&lights.len()),
            );
            let light_uniform_size = size_of::<Light>() as wgpu::BufferAddress;
            for (i, (color, radius, transform)) in lights.into_iter().enumerate() {
                let offset = (light_uniform_size * i as u64) as wgpu::BufferAddress;
                let position = transform * Vec4::W;
                let trunk = Light {
                    position: [position.x, position.y, position.z],
                    radius: radius,
                    color: [
                        color.r as f32,
                        color.g as f32,
                        color.b as f32,
                        color.a as f32,
                    ],
                };
                self.queue
                    .write_buffer(&self.light_buffer, offset, bytemuck::bytes_of(&trunk));
            }
            let node_uniform_aligned = {
                let node_uniform_size = size_of::<Mat4>() as wgpu::BufferAddress;
                let alignment =
                    self.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
                align_to(node_uniform_size, alignment)
            };
            for (i, (geometry, _shader, transform)) in nodes.iter().enumerate() {
                let offset = (node_uniform_aligned * i as u64) as wgpu::BufferAddress;
                self.queue.write_buffer(
                    &self.w_buffer,
                    offset,
                    bytemuck::cast_slice(transform.as_ref()),
                );
                rpass.set_bind_group(0, &self.bind_group_node, &[offset as wgpu::DynamicOffset]);
                rpass.set_index_buffer(geometry.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
                let n = geometry.indices.len() as u32;
                rpass.draw_indexed(0..n, 0, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
