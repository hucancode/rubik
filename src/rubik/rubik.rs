use crate::geometry::Mesh;
use crate::material::Shader;
use crate::world::{new_entity, new_group, Node, NodeRef};
use std::f32::consts::PI;
use std::sync::Arc;
use tween::{SineInOut, Tweener};

enum Move {
    TOP(usize),
    BOTTOM(usize),
    LEFT(usize),
    RIGHT(usize),
    FRONT(usize),
    BACK(usize),
    NONE,
}
pub struct Rubik {
    pub pieces: Vec<Vec<Vec<NodeRef>>>,
    tween: Tweener<f32, f32, SineInOut>,
    current_move: Move,
}
const CUBE_SIZE: f32 = 2.0;
const CUBE_MARGIN: f32 = 0.15;
impl Rubik {
    pub fn new() -> Self {
        Self {
            pieces: Vec::new(),
            tween: Tweener::sine_in_out(0.0, PI * 2.0, 500.0),
            current_move: Move::LEFT(1),
        }
    }
    pub fn generate_pieces(&mut self, dimension: usize, shader: Arc<Shader>, mesh: Arc<Mesh>) {
        let d = CUBE_SIZE + CUBE_MARGIN;
        let n = dimension as i32;
        self.pieces.clear();
        for z in -n..=n {
            let mut row = new_group();
            let mut pxy = Vec::new();
            for y in -n..=n {
                let mut px = Vec::new();
                for x in -n..=n {
                    let mut cube = new_entity(mesh.clone(), shader.clone());
                    px.push(cube.clone());
                    row.add_child(cube.clone());
                    cube.translate(d * x as f32, d * y as f32, d * z as f32);
                }
                pxy.push(px);
            }
            self.pieces.push(pxy);
        }
    }
    pub fn update(&mut self, delta_time: f32) {
        println!("rubik update {delta_time}");
        let alpha = self.tween.move_by(delta_time);
        match self.current_move {
            Move::TOP(depth) => {
                let mat = glam::Mat4::from_rotation_z(alpha);
                for (_, pz) in self
                    .pieces
                    .iter_mut()
                    .enumerate()
                    .filter(|(z, _)| *z < depth)
                {
                    for (_, py) in pz.iter_mut().enumerate() {
                        for (_, piece) in py.iter_mut().enumerate() {
                            let transform = mat * piece.calculate_transform();
                            let (_scale, rotation, translation) =
                                transform.to_scale_rotation_translation();
                            piece.translate(translation.x, translation.y, translation.z);
                            piece.rotate_quat(rotation);
                        }
                    }
                }
                if self.tween.is_finished() {
                    self.tween = Tweener::sine_in_out(0.0, PI * 2.0, 50.0);
                }
            }
            Move::LEFT(depth) => {
                let mat = glam::Mat4::from_rotation_x(alpha);
                for (_, pz) in self.pieces.iter_mut().enumerate() {
                    for (_, py) in pz.iter_mut().enumerate() {
                        for (_, piece) in py.iter_mut().enumerate().filter(|(x, _)| *x < depth) {
                            let transform = mat * piece.calculate_transform();
                            let (_scale, rotation, translation) =
                                transform.to_scale_rotation_translation();
                            piece.translate(translation.x, translation.y, translation.z);
                            piece.rotate_quat(rotation);
                        }
                    }
                }
                if self.tween.is_finished() {
                    self.tween = Tweener::sine_in_out(0.0, PI * 2.0, 50.0);
                }
            }
            _ => {}
        };
    }
}
