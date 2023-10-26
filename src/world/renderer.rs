use winit::window::Window;
use wgpu::{Surface, 
	SurfaceConfiguration, 
	Device,
	Queue,
	BindGroup,
	RenderPipeline,
	Buffer};
use wgpu::util::DeviceExt;
use std::collections::VecDeque;
use crate::shader::Shader;
use crate::world::Camera;
use crate::world::Node;
use crate::geometry::Vertex;
pub struct Renderer {
	pub camera: Camera,
	pub root: Node,
	config: SurfaceConfiguration,
	surface: Surface,
	pub device: Device,
	queue: Queue,
	render_pipeline: RenderPipeline,
	bind_group: BindGroup,
	uniform_buffer: Buffer,
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

		let shader = Shader::new(&device, include_str!("../shader/shader.wgsl"));

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

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: swapchain_format,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: swapchain_capabilities.alpha_modes[0],
			view_formats: vec![],
		};

		let mvp = Camera::make_vp_matrix(config.width as f32 / config.height as f32);
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
		Self {
			camera: Camera::new(),
			root: Node::new_empty(),
			config,
			surface,
			device,
			queue,
			render_pipeline,
			bind_group,
			uniform_buffer,
		}
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self.config.width = width;
		self.config.height = height;
		self.surface.configure(&self.device, &self.config);
		let mvp = Camera::make_vp_matrix(self.config.width as f32 / self.config.height as f32);
		let mvp_ref: &[f32; 16] = mvp.as_ref();
		self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(mvp_ref));
	}

	pub fn draw(&self) {
		let frame = self.surface
			.get_current_texture()
			.expect("Failed to acquire next swap chain texture");
		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder =
			self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
			rpass.set_bind_group(0, &self.bind_group, &[]);
			let mut q = VecDeque::new();
			q.push_back(&self.root);
			while let Some(node) = q.pop_front() {
				if let Some(visual) = &node.visual {
					rpass.set_index_buffer(visual.geometry.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
					rpass.set_vertex_buffer(0, visual.geometry.vertex_buffer.slice(..));
					let n = visual.geometry.indices.len() as u32;
					rpass.draw_indexed(0..n, 0, 0..1);
				}
				for child in node.children.iter() {
					q.push_back(child);
				}
			}
		}
		self.queue.submit(Some(encoder.finish()));
		frame.present();
	}
}