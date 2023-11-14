use crate::geometry::Mesh;
use crate::material::Shader;
use crate::world::{new_entity, new_group, Node, NodeRef};
use std::f32::consts::PI;
use std::sync::Arc;
pub struct Rubik {
    pub rows: Vec<NodeRef>,
}
const CUBE_SIZE: f32 = 2.0;
const CUBE_MARGIN: f32 = 0.15;
impl Rubik {
    pub fn new() -> Self {
        Self { rows: Vec::new() }
    }
    pub fn generate_pieces(&mut self, dimension: usize, shader: Arc<Shader>, mesh: Arc<Mesh>) {
        self.rows.clear();
        let d = CUBE_SIZE + CUBE_MARGIN;
        let n = dimension as i32;
        for z in -n..=n {
            let mut row = new_group();
            for y in -n..=n {
                for x in -n..=n {
                    let mut cube = new_entity(mesh.clone(), shader.clone());
                    row.add_child(cube.clone());
                    cube.translate(d * x as f32, d * y as f32, d * z as f32);
                }
            }
            self.rows.push(row.clone());
        }
    }
    pub fn update(&mut self, time: u128) {
        for (i, row) in self.rows.iter_mut().enumerate() {
            let alpha = PI * (1.0 + ((time as f64) * 0.0007 + (i as f64) * 0.08).sin() as f32);
            row.rotate_z(alpha);
        }
    }
}
