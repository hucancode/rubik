use bytemuck::{Pod, Zeroable};
use wgpu::{
	BufferAddress,
	VertexStepMode,
	VertexBufferLayout,
	VertexAttribute,
	vertex_attr_array};
use std::mem::size_of;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}
impl Vertex {
	pub fn new(pos: [f32; 3], col: u32) -> Self {
		let x = pos[0];
		let y = pos[1];
		let z = pos[2];
		let w = 1.0;
		let r = 0xff & (col >> 24);
		let g = 0xff & (col >> 16);
		let b = 0xff & (col >> 8);
		let a = 0xff & col;
		Self {
			position: [x, y, z, w],
			color: [
				r as f32 / 255.0,
				g as f32 / 255.0,
				b as f32 / 255.0,
				a as f32 / 255.0,
			],
		}
	}
	pub fn desc() -> VertexBufferLayout<'static> {
        const ATTRIBS: [VertexAttribute; 2] =
            vertex_attr_array![0 => Float32x4, 1 => Float32x4];
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}