use bytemuck::{Pod, Zeroable};
use std::mem::size_of;
use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub color: [f32; 4],
}
impl Vertex {
    pub fn new(pos: [f32; 3], nor: [f32; 3], col: u32) -> Self {
        let x = pos[0];
        let y = pos[1];
        let z = pos[2];
        let w = 1.0;
        let nx = nor[0];
        let ny = nor[1];
        let nz = nor[2];
        let nw = 1.0;
        let r = 0xff & (col >> 24);
        let g = 0xff & (col >> 16);
        let b = 0xff & (col >> 8);
        let a = 0xff & col;
        Self {
            position: [x, y, z, w],
            normal: [nx, ny, nz, nw],
            color: [
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            ],
        }
    }
    pub fn desc() -> VertexBufferLayout<'static> {
        const ATTRIBS: [VertexAttribute; 3] =
            vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4];
        VertexBufferLayout {
            array_stride: size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
}
