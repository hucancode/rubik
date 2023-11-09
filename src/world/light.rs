use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Light {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 4],
}
