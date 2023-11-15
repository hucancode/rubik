use crate::geometry::Mesh;
use crate::material::Shader;
use crate::world::{new_entity, new_group, Node, NodeRef};
use rand::Rng;
use std::convert::From;
use std::f32::consts::PI;
use std::sync::Arc;
use tween::{SineInOut, Tweener};

#[derive(Clone, Copy)]
pub enum Move {
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,
    FRONT,
    BACK,
    NONE,
}

impl From<i32> for Move {
    fn from(v: i32) -> Self {
        match v {
            0 => Move::TOP,
            1 => Move::BOTTOM,
            2 => Move::LEFT,
            3 => Move::RIGHT,
            4 => Move::FRONT,
            5 => Move::BACK,
            _ => Move::NONE,
        }
    }
}
pub struct Rubik {
    pieces: Vec<NodeRef>,
    tween: Tweener<f32, f32, SineInOut>,
    current_move: Move,
    moving_pieces: Vec<NodeRef>,
    pub root: NodeRef,
    moving_pivot: NodeRef,
    static_root: NodeRef,
    span: usize,
}
const CUBE_SIZE: f32 = 2.0;
const CUBE_MARGIN: f32 = 0.15;
impl Rubik {
    pub fn new() -> Self {
        let moving_pivot = new_group();
        let static_root = new_group();
        let mut root = new_group();
        root.add_child(moving_pivot.clone());
        root.add_child(static_root.clone());
        Self {
            pieces: Vec::new(),
            tween: Tweener::sine_in_out(0.0, PI * 2.0, 5.0),
            current_move: Move::NONE,
            moving_pieces: Vec::new(),
            root,
            moving_pivot,
            static_root,
            span: 0,
        }
    }
    pub fn generate_pieces(&mut self, span: usize, shader: Arc<Shader>, mesh: Arc<Mesh>) {
        let d = CUBE_SIZE + CUBE_MARGIN;
        self.span = span;
        let n = span as i32;
        self.pieces.clear();
        for z in -n..=n {
            for y in -n..=n {
                for x in -n..=n {
                    let mut cube = new_entity(mesh.clone(), shader.clone());
                    self.pieces.push(cube.clone());
                    self.static_root.add_child(cube.clone());
                    cube.translate(d * x as f32, d * y as f32, d * z as f32);
                }
            }
        }
    }
    pub fn start_move_random(&mut self) {
        let mut rng = rand::thread_rng();
        self.current_move = Move::from(rng.gen_range(0..6));
        let size = CUBE_SIZE + CUBE_MARGIN;
        let depth = rng.gen_range(0..self.span * 2) as f32 * size * 0.5;
        match self.current_move {
            Move::TOP => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().z;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::BOTTOM => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().z;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::LEFT => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().x;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::RIGHT => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().x;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::FRONT => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().y;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::BACK => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().y;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            _ => {}
        };
        self.tween = Tweener::sine_in_out(0.0, PI * 2.0, 2.0);
    }
    pub fn finish_move(&mut self) {
        let mat = self.moving_pivot.calculate_transform();
        for (_, piece) in self.moving_pieces.iter_mut().enumerate() {
            let transform = mat * piece.calculate_transform();
            let (_scale, rotation, translation) = transform.to_scale_rotation_translation();
            piece.translate(translation.x, translation.y, translation.z);
            piece.rotate_quat(rotation);
        }
        for piece in self.moving_pivot.extract_child() {
            self.static_root.add_child(piece);
        }
        self.moving_pivot.rotate(0.0, 0.0, 0.0);
        self.start_move_random();
    }
    pub fn update(&mut self, delta_time: f32) {
        let alpha = self.tween.move_by(delta_time);
        match self.current_move {
            Move::TOP => {
                self.moving_pivot.rotate_z(alpha);
            }
            Move::BOTTOM => {
                self.moving_pivot.rotate_z(alpha);
            }
            Move::LEFT => {
                self.moving_pivot.rotate_x(alpha);
            }
            Move::RIGHT => {
                self.moving_pivot.rotate_x(alpha);
            }
            Move::FRONT => {
                self.moving_pivot.rotate_y(alpha);
            }
            Move::BACK => {
                self.moving_pivot.rotate_y(alpha);
            }
            _ => {}
        };
        if self.tween.is_finished() {
            self.finish_move();
        }
    }
}
