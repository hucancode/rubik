use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::borrow::Cow;
use std::f32::consts;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 4],
    color: [f32; 4],
}
impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBS: [wgpu::VertexAttribute; 2] =
            wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}
fn make_vertex(pos: [f32; 3], col: u32) -> Vertex {
    let x = pos[0];
    let y = pos[1];
    let z = pos[2];
    let w = 1.0;
    let r = 0xff & (col >> 24);
    let g = 0xff & (col >> 16);
    let b = 0xff & (col >> 8);
    let a = 0xff & col;
    Vertex {
        position: [x, y, z, w],
        color: [
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        ],
    }
}

fn make_cube_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let green = 0x40a02bff; // right - green
    let purple = 0x89b4faff; // left - purple
    let yellow = 0xf9e2afff; // top - yellow
    let white = 0xf8fafcff; // bottom - white
    let red = 0xef4444ff; // front - red
    let orange = 0xfe640bff; // back - orange
    let vertex_data = [
        // top (0, 0, 1)
        make_vertex([-1.0, -1.0, 1.0], yellow),
        make_vertex([1.0, -1.0, 1.0], yellow),
        make_vertex([1.0, 1.0, 1.0], yellow),
        make_vertex([-1.0, 1.0, 1.0], yellow),
        // bottom (0, 0, -1.0)
        make_vertex([-1.0, 1.0, -1.0], white),
        make_vertex([1.0, 1.0, -1.0], white),
        make_vertex([1.0, -1.0, -1.0], white),
        make_vertex([-1.0, -1.0, -1.0], white),
        // right (1, 0, 0)
        make_vertex([1.0, -1.0, -1.0], green),
        make_vertex([1.0, 1.0, -1.0], green),
        make_vertex([1.0, 1.0, 1.0], green),
        make_vertex([1.0, -1.0, 1.0], green),
        // left (-1, 0, 0)
        make_vertex([-1.0, -1.0, 1.0], purple),
        make_vertex([-1.0, 1.0, 1.0], purple),
        make_vertex([-1.0, 1.0, -1.0], purple),
        make_vertex([-1.0, -1.0, -1.0], purple),
        // front (0, 1.0, 0)
        make_vertex([1.0, 1.0, -1.0], red),
        make_vertex([-1.0, 1.0, -1.0], red),
        make_vertex([-1.0, 1.0, 1.0], red),
        make_vertex([1.0, 1.0, 1.0], red),
        // back (0, -1.0, 0)
        make_vertex([1.0, -1.0, 1.0], orange),
        make_vertex([-1.0, -1.0, 1.0], orange),
        make_vertex([-1.0, -1.0, -1.0], orange),
        make_vertex([1.0, -1.0, -1.0], orange),
    ];
    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];
    (vertex_data.to_vec(), index_data.to_vec())
}
fn make_mvp_matrix(aspect_ratio: f32) -> Mat4 {
    let projection = Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
    let view = Mat4::look_at_rh(Vec3::new(1.5f32, -5.0, 3.0), Vec3::ZERO, Vec3::Z);
    projection * view
}

pub async fn run(event_loop: EventLoop<()>, window: Window) {
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

    // Load the shaders from disk
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    let (vertex_data, index_data) = make_cube_vertices();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertex_data),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(&index_data),
        usage: wgpu::BufferUsages::INDEX,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(64),
            },
            count: None,
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
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

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };

    let mvp = make_mvp_matrix(config.width as f32 / config.height as f32);
    let mvp_ref: &[f32; 16] = mvp.as_ref();
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(mvp_ref),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    surface.configure(&device, &config);

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: None,
    });

    event_loop.run(move |event, _, control_flow| {
        let _ = (&instance, &adapter, &shader, &pipeline_layout);
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                let mvp = make_mvp_matrix(config.width as f32 / config.height as f32);
                let mvp_ref: &[f32; 16] = mvp.as_ref();
                queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(mvp_ref));
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    let n = index_data.len() as u32;
                    rpass.draw_indexed(0..n, 0, 0..1);
                }
                queue.submit(Some(encoder.finish()));
                frame.present();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
