use crate::geometry::Vertex;
use crate::material::Shader;
use crate::world::{node, Camera, Light, Node, NodeRef};
use glam::{Mat4, Vec4};
use std::cmp::max;
use std::mem::size_of;
use std::time::Instant;
use wgpu::util::{align_to, BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Color, CommandEncoderDescriptor,
    CompareFunction, DepthBiasState, DepthStencilState, Device, DeviceDescriptor, DynamicOffset,
    Extent3d, Face, Features, FragmentState, FrontFace, IndexFormat, Instance, Limits, LoadOp,
    MultisampleState, Operations, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderStages, StencilState, StoreOp, Surface, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
    VertexState,
};
use winit::window::Window;

const MAX_ENTITY: u64 = 100000;
const MAX_LIGHT: u64 = 10;
const CLEAR_COLOR: Color = Color {
    r: 0.00633333333,
    g: 0.00633333333,
    b: 0.01388235294,
    a: 1.0,
};
const CAMERA_DISTANCE: f32 = 10.0;

pub struct Renderer {
    pub camera: Camera,
    pub root: NodeRef,
    config: SurfaceConfiguration,
    surface: Surface,
    pub device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    depth_texture_view: TextureView,
    bind_group_camera: BindGroup,
    bind_group_node: BindGroup,
    vp_buffer: Buffer,
    w_buffer: Buffer,
    r_buffer: Buffer,
    light_buffer: Buffer,
    light_count_buffer: Buffer,
}

impl Renderer {
    pub async fn new(window: &Window) -> Renderer {
        let new_renderer_timestamp = Instant::now();
        let size = window.inner_size();
        let instance = Instance::default();
        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        println!("created surface in {:?}", new_renderer_timestamp.elapsed());
        let device_request_timestamp = Instant::now();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");
        println!(
            "requested device in {:?}",
            device_request_timestamp.elapsed()
        );
        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: swapchain_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let new_shader_timestamp = Instant::now();
        let bind_group_layout_camera =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0, // view projection
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(size_of::<Mat4>() as u64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1, // light
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(0),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2, // light count
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: BufferSize::new(size_of::<usize>() as u64),
                        },
                        count: None,
                    },
                ],
            });
        let bind_group_layout_node = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0, // world
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(size_of::<Mat4>() as u64),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1, // rotation
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(size_of::<Mat4>() as u64),
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout_node, &bind_group_layout_camera],
            push_constant_ranges: &[],
        });

        let shader = Shader::new(&device, include_str!("../material/shader.wgsl"));
        println!("created shader in {:?}", new_shader_timestamp.elapsed());
        let new_pipeline_timestamp = Instant::now();

        let depth_texture = device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&TextureViewDescriptor::default());
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader.module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader.module,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
        });
        let vp =
            Camera::make_vp_matrix(config.width as f32 / config.height as f32, CAMERA_DISTANCE);
        let vp_ref: &[f32; 16] = vp.as_ref();
        let vp_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera View Projection Buffer"),
            contents: bytemuck::cast_slice(vp_ref),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let light_uniform_size = size_of::<Light>() as BufferAddress;
        let light_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Light Buffer"),
            size: MAX_LIGHT as BufferAddress * light_uniform_size,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_count_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Light Count"),
            size: size_of::<usize>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_camera = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_camera,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: vp_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: light_count_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        let node_uniform_size = size_of::<Mat4>() as BufferAddress;
        let node_uniform_aligned = {
            let alignment = device.limits().min_uniform_buffer_offset_alignment as BufferAddress;
            align_to(node_uniform_size, alignment)
        };
        let w_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Model world transform buffer"),
            size: MAX_ENTITY as BufferAddress * node_uniform_aligned,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let r_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Model rotation buffer"),
            size: MAX_ENTITY as BufferAddress * node_uniform_aligned,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_node = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout_node,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &w_buffer,
                        offset: 0,
                        size: BufferSize::new(node_uniform_size),
                    }),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &r_buffer,
                        offset: 0,
                        size: BufferSize::new(node_uniform_size),
                    }),
                },
            ],
            label: None,
        });
        println!("created pipeline in {:?}", new_pipeline_timestamp.elapsed());
        println!(
            "in total, created new renderer in {:?}",
            new_renderer_timestamp.elapsed()
        );
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
            r_buffer,
            light_buffer,
            light_count_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = max(1, width);
        self.config.height = max(1, height);
        self.surface.configure(&self.device, &self.config);
        let mvp = Camera::make_vp_matrix(
            self.config.width as f32 / self.config.height as f32,
            CAMERA_DISTANCE,
        );
        let mvp_ref: &[f32; 16] = mvp.as_ref();
        self.queue
            .write_buffer(&self.vp_buffer, 0, bytemuck::cast_slice(mvp_ref));
        let depth_texture = self.device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });
        self.depth_texture_view = depth_texture.create_view(&TextureViewDescriptor::default());
    }

    pub fn draw(&self) {
        let frame = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });
        let mut nodes = Vec::new();
        let mut lights: Vec<(Color, f32, Mat4)> = Vec::new();
        {
            nodes.clear();
            lights.clear();
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(CLEAR_COLOR),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Discard,
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
                match &node.borrow().variant {
                    node::Variant::Entity(geometry, shader) => {
                        let (_scale, rotation, _translation) =
                            transform_mx.to_scale_rotation_translation();
                        let rotation = Mat4::from_quat(rotation);
                        nodes.push((geometry.clone(), shader.clone(), transform_mx, rotation));
                    }
                    node::Variant::Light(color, radius) => {
                        lights.push((*color, *radius, transform_mx));
                    }
                    _ => {}
                }
                for child in node.borrow().children.iter() {
                    let transform_mx = transform_mx * child.calculate_transform();
                    q.push((child.clone(), transform_mx));
                }
            }
            self.queue.write_buffer(
                &self.light_count_buffer,
                0,
                bytemuck::bytes_of(&lights.len()),
            );
            let light_uniform_size = size_of::<Light>() as BufferAddress;
            for (i, (color, radius, transform)) in lights.into_iter().enumerate() {
                let offset = (light_uniform_size * i as u64) as BufferAddress;
                let position = transform * Vec4::W;
                let trunk = Light {
                    position: [position.x, position.y, position.z],
                    radius,
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
                let node_uniform_size = size_of::<Mat4>() as BufferAddress;
                let alignment =
                    self.device.limits().min_uniform_buffer_offset_alignment as BufferAddress;
                align_to(node_uniform_size, alignment)
            };
            for (i, (geometry, _shader, transform, rotation)) in nodes.iter().enumerate() {
                let offset = (node_uniform_aligned * i as u64) as BufferAddress;
                self.queue.write_buffer(
                    &self.w_buffer,
                    offset,
                    bytemuck::cast_slice(transform.as_ref()),
                );
                self.queue.write_buffer(
                    &self.r_buffer,
                    offset,
                    bytemuck::cast_slice(rotation.as_ref()),
                );
                rpass.set_bind_group(
                    0,
                    &self.bind_group_node,
                    &[offset as DynamicOffset, offset as DynamicOffset],
                );
                rpass.set_index_buffer(geometry.index_buffer.slice(..), IndexFormat::Uint16);
                rpass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
                let n = geometry.indices.len() as u32;
                rpass.draw_indexed(0..n, 0, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
