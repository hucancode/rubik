use crate::geometry::Vertex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device};

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u16>, device: &Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }
}
