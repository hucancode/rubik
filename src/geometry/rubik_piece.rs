use crate::geometry::Geometry;
use crate::geometry::Vertex;
use wgpu::Device;

impl Geometry {
    pub fn new_rubik_piece(device: &Device) -> Self {
        let green = 0x40a02bff; // right - green
        let purple = 0x89b4faff; // left - purple
        let yellow = 0xf9e2afff; // top - yellow
        let white = 0xf8fafcff; // bottom - white
        let red = 0xef4444ff; // front - red
        let orange = 0xfe640bff; // back - orange
        let vertex_data = [
            // top (0, 0, 1)
            Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0], yellow),
            Vertex::new([1.0, -1.0, 1.0], [0.0, 0.0, 1.0], yellow),
            Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], yellow),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 0.0, 1.0], yellow),
            // bottom (0, 0, -1.0)
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 0.0, -1.0], white),
            Vertex::new([1.0, 1.0, -1.0], [0.0, 0.0, -1.0], white),
            Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0, -1.0], white),
            Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, -1.0], white),
            // right (1, 0, 0)
            Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 0.0], green),
            Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0, 0.0], green),
            Vertex::new([1.0, 1.0, 1.0], [1.0, 0.0, 0.0], green),
            Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 0.0], green),
            // left (-1, 0, 0)
            Vertex::new([-1.0, -1.0, 1.0], [-1.0, 0.0, 0.0], purple),
            Vertex::new([-1.0, 1.0, 1.0], [-1.0, 0.0, 0.0], purple),
            Vertex::new([-1.0, 1.0, -1.0], [-1.0, 0.0, 0.0], purple),
            Vertex::new([-1.0, -1.0, -1.0], [-1.0, 0.0, 0.0], purple),
            // front (0, 1.0, 0)
            Vertex::new([1.0, 1.0, -1.0], [0.0, 1.0, 0.0], red),
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0], red),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 0.0], red),
            Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 0.0], red),
            // back (0, -1.0, 0)
            Vertex::new([1.0, -1.0, 1.0], [0.0, -1.0, 0.0], orange),
            Vertex::new([-1.0, -1.0, 1.0], [0.0, -1.0, 0.0], orange),
            Vertex::new([-1.0, -1.0, -1.0], [0.0, -1.0, 0.0], orange),
            Vertex::new([1.0, -1.0, -1.0], [0.0, -1.0, 0.0], orange),
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
