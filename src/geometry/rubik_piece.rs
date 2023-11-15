use crate::geometry::Mesh;
use crate::geometry::Vertex;
use wgpu::Device;

impl Mesh {
    pub fn new_rubik_piece(
        device: &Device,
        faced_top: bool,
        faced_bottom: bool,
        faced_left: bool,
        faced_right: bool,
        faced_front: bool,
        faced_back: bool,
    ) -> Self {
        const GREEN: u32 = 0x40a02bff; // right - green
        const PURPLE: u32 = 0x89b4faff; // left - purple
        const YELLOW: u32 = 0xf9e2afff; // top - yellow
        const WHITE: u32 = 0xf8fafcff; // bottom - white
        const RED: u32 = 0xef4444ff; // front - red
        const ORANGE: u32 = 0xfe640bff; // back - orange
        const BLACK: u32 = 0x040407ff; // black
        let top_color = if faced_top { YELLOW } else { BLACK };
        let bottom_color = if faced_bottom { WHITE } else { BLACK };
        let left_color = if faced_left { PURPLE } else { BLACK };
        let right_color = if faced_right { GREEN } else { BLACK };
        let front_color = if faced_front { RED } else { BLACK };
        let back_color = if faced_back { ORANGE } else { BLACK };
        let vertex_data = [
            // top (0, 0, 1)
            Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0], top_color),
            Vertex::new([1.0, -1.0, 1.0], [0.0, 0.0, 1.0], top_color),
            Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0, 1.0], top_color),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 0.0, 1.0], top_color),
            // bottom (0, 0, -1.0)
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 0.0, -1.0], bottom_color),
            Vertex::new([1.0, 1.0, -1.0], [0.0, 0.0, -1.0], bottom_color),
            Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0, -1.0], bottom_color),
            Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, -1.0], bottom_color),
            // right (1, 0, 0)
            Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 0.0], right_color),
            Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0, 0.0], right_color),
            Vertex::new([1.0, 1.0, 1.0], [1.0, 0.0, 0.0], right_color),
            Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 0.0], right_color),
            // left (-1, 0, 0)
            Vertex::new([-1.0, -1.0, 1.0], [-1.0, 0.0, 0.0], left_color),
            Vertex::new([-1.0, 1.0, 1.0], [-1.0, 0.0, 0.0], left_color),
            Vertex::new([-1.0, 1.0, -1.0], [-1.0, 0.0, 0.0], left_color),
            Vertex::new([-1.0, -1.0, -1.0], [-1.0, 0.0, 0.0], left_color),
            // front (0, 1.0, 0)
            Vertex::new([1.0, 1.0, -1.0], [0.0, 1.0, 0.0], front_color),
            Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0], front_color),
            Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 0.0], front_color),
            Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 0.0], front_color),
            // back (0, -1.0, 0)
            Vertex::new([1.0, -1.0, 1.0], [0.0, -1.0, 0.0], back_color),
            Vertex::new([-1.0, -1.0, 1.0], [0.0, -1.0, 0.0], back_color),
            Vertex::new([-1.0, -1.0, -1.0], [0.0, -1.0, 0.0], back_color),
            Vertex::new([1.0, -1.0, -1.0], [0.0, -1.0, 0.0], back_color),
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
