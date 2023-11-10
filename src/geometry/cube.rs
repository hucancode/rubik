use crate::geometry::Mesh;
use crate::geometry::Vertex;
use wgpu::Device;

impl Mesh {
    pub fn new_cube(col: u32, device: &Device) -> Self {
        let vertex_data = [
            // top (0, 0, 1)
            Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0], col),
            Vertex::new([1.0, -1.0, 1.0], [0.0, 0.0, 1.0], col),
            Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], col),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 0.0, 1.0], col),
            // bottom (0, 0, -1.0)
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 0.0, -1.0], col),
            Vertex::new([1.0, 1.0, -1.0], [0.0, 0.0, -1.0], col),
            Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0, -1.0], col),
            Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, -1.0], col),
            // right (1, 0, 0)
            Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 0.0], col),
            Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0, 0.0], col),
            Vertex::new([1.0, 1.0, 1.0], [1.0, 0.0, 0.0], col),
            Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 0.0], col),
            // left (-1, 0, 0)
            Vertex::new([-1.0, -1.0, 1.0], [-1.0, 0.0, 0.0], col),
            Vertex::new([-1.0, 1.0, 1.0], [-1.0, 0.0, 0.0], col),
            Vertex::new([-1.0, 1.0, -1.0], [-1.0, 0.0, 0.0], col),
            Vertex::new([-1.0, -1.0, -1.0], [-1.0, 0.0, 0.0], col),
            // front (0, 1.0, 0)
            Vertex::new([1.0, 1.0, -1.0], [0.0, 1.0, 0.0], col),
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0], col),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 0.0], col),
            Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 0.0], col),
            // back (0, -1.0, 0)
            Vertex::new([1.0, -1.0, 1.0], [0.0, -1.0, 0.0], col),
            Vertex::new([-1.0, -1.0, 1.0], [0.0, -1.0, 0.0], col),
            Vertex::new([-1.0, -1.0, -1.0], [0.0, -1.0, 0.0], col),
            Vertex::new([1.0, -1.0, -1.0], [0.0, -1.0, 0.0], col),
        ];
        let index_data: &[u16] = &[
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];
        Self::new(vertex_data.to_vec(), index_data.to_vec(), device)
    }
}
