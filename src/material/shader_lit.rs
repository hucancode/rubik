use crate::material::Shader;
use glam::Mat4;
use std::borrow::Cow;
use std::mem::size_of;
use std::time::Instant;
use wgpu::util::{align_to, BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferAddress, BufferBinding,
    BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, CompareFunction, DepthBiasState,
    DepthStencilState, DynamicOffset, Face, FragmentState, FrontFace, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderStages, StencilState, TextureFormat, VertexState,
};

use crate::geometry::Vertex;
use crate::world::{Light, Renderer};

const MAX_ENTITY: u64 = 100000;
const MAX_LIGHT: u64 = 10;
pub struct ShaderLit {
    pub module: ShaderModule,
    pub render_pipeline: RenderPipeline,
    pub bind_group_camera: BindGroup,
    pub bind_group_node: BindGroup,
    pub vp_buffer: Buffer,
    pub w_buffer: Buffer,
    pub r_buffer: Buffer,
    pub light_buffer: Buffer,
    pub light_count_buffer: Buffer,
}
impl ShaderLit {
    pub fn new(renderer: &Renderer) -> Self {
        let device = &renderer.device;
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
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader_lit.wgsl"))),
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &module,
                entry_point: "fs_main",
                targets: &[Some(renderer.config.format.into())],
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
        let vp_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera View Projection Buffer"),
            contents: bytemuck::cast_slice(Mat4::IDENTITY.as_ref()),
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
                    binding: 0, // world transform
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &w_buffer,
                        offset: 0,
                        size: BufferSize::new(node_uniform_size),
                    }),
                },
                BindGroupEntry {
                    binding: 1, // rotation
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &r_buffer,
                        offset: 0,
                        size: BufferSize::new(node_uniform_size),
                    }),
                },
            ],
            label: None,
        });
        println!("created shader in {:?}", new_shader_timestamp.elapsed());
        Self {
            module,
            render_pipeline,
            bind_group_camera,
            bind_group_node,
            vp_buffer,
            w_buffer,
            r_buffer,
            light_buffer,
            light_count_buffer,
        }
    }
}
impl Shader for ShaderLit {
    fn set_pipeline<'a>(&'a self, pass: &mut RenderPass<'a>, offset: BufferAddress) {
        let offsets = [offset as DynamicOffset, offset as DynamicOffset];
        pass.set_bind_group(0, &self.bind_group_node, &offsets);
        pass.set_bind_group(1, &self.bind_group_camera, &[]);
        pass.set_pipeline(&self.render_pipeline);
    }
    fn write_transform_data(&self, queue: &Queue, offset: BufferAddress, matrix: &[f32; 16]) {
        queue.write_buffer(&self.w_buffer, offset, bytemuck::bytes_of(matrix));
    }
    fn write_rotation_data(&self, queue: &Queue, offset: BufferAddress, matrix: &[f32; 16]) {
        queue.write_buffer(&self.r_buffer, offset, bytemuck::bytes_of(matrix));
    }
    fn write_time_data(&self, _queue: &Queue, _time: f32) {
        // do nothing
    }
    fn write_camera_data(&self, queue: &Queue, matrix: &[f32; 16]) {
        queue.write_buffer(&self.vp_buffer, 0, bytemuck::bytes_of(matrix));
    }
    fn write_light_data(&self, queue: &Queue, lights: &Vec<Light>) {
        queue.write_buffer(
            &self.light_count_buffer,
            0,
            bytemuck::bytes_of(&lights.len()),
        );
        queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(lights));
    }
}
