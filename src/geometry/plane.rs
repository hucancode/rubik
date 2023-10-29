use crate::geometry::Vertex;
use crate::geometry::Geometry;
use wgpu::Device;

impl Geometry {
	pub fn new_plane(col: u32, device: &Device) -> Self {
        let vertex_data = [
            vertex([size, -size, 0], [0, 0, 1]),
            vertex([size, size, 0], [0, 0, 1]),
            vertex([-size, -size, 0], [0, 0, 1]),
            vertex([-size, size, 0], [0, 0, 1]),
        ];

        let index_data: &[u16] = &[0, 1, 2, 2, 1, 3];
		Self::new(vertex_data.to_vec(), index_data.to_vec(), device)
    }
}