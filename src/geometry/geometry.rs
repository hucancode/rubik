use wgpu::{Device, Buffer, util::DeviceExt};
use crate::geometry::Vertex;

pub struct Geometry {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u16>,
	pub vertex_buffer: Buffer,
	pub index_buffer: Buffer,
}

impl Geometry {
	pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>, device: &Device) -> Self {
		let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytemuck::cast_slice(&vertices),
			usage: wgpu::BufferUsages::VERTEX,
		});
		let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(&indices),
			usage: wgpu::BufferUsages::INDEX,
		});
		Self { vertices, indices, vertex_buffer, index_buffer }
	}
}