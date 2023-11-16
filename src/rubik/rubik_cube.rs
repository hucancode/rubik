use crate::geometry::Mesh;
use crate::material::Shader;
use crate::rubik::Move;
use crate::world::{new_entity, new_group, Node, NodeRef};
use rand::Rng;
use std::f32::consts::PI;
use std::sync::Arc;
use tween::{
    BackIn, BackInOut, BackOut, BounceIn, BounceInOut, BounceOut, CircIn, CircInOut, CircOut,
    CubicIn, CubicInOut, CubicOut, ElasticIn, ElasticInOut, ElasticOut, ExpoIn, ExpoInOut, ExpoOut,
    Linear, QuadIn, QuadInOut, QuadOut, QuintIn, QuintInOut, QuintOut, SineIn, SineInOut, SineOut,
    Tween, Tweener,
};
use wgpu::Device;

const CUBE_SIZE: f32 = 2.0;
const CUBE_MARGIN: f32 = 0.15;

type SendSyncTween<Value, Time> = Tweener<Value, Time, Box<dyn Tween<Value> + Send + Sync>>;

pub struct Rubik {
    pieces: Vec<NodeRef>,
    tween: SendSyncTween<f32, f32>,
    current_move: Move,
    pub root: NodeRef,
    moving_pivot: NodeRef,
    static_root: NodeRef,
    span: usize,
}

impl Rubik {
    pub fn new() -> Self {
        let moving_pivot = new_group();
        let static_root = new_group();
        let mut root = new_group();
        root.add_child(moving_pivot.clone());
        root.add_child(static_root.clone());
        Self {
            pieces: Vec::new(),
            tween: Tweener::new(0.0, PI * 2.0, 5.0, Box::new(Linear)),
            current_move: Move::None,
            root,
            moving_pivot,
            static_root,
            span: 0,
        }
    }
    pub fn generate_pieces(&mut self, span: usize, device: &Device) {
        let shader = Arc::new(Shader::new(device, include_str!("../material/shader.wgsl")));
        let d = CUBE_SIZE + CUBE_MARGIN;
        self.span = span;
        let n = span as i32;
        self.pieces.clear();
        for z in -n..=n {
            for y in -n..=n {
                for x in -n..=n {
                    let faced_top = z == n;
                    let faced_bottom = z == -n;
                    let faced_left = x == -n;
                    let faced_right = x == n;
                    let faced_front = y == n;
                    let faced_back = y == -n;
                    let rubik_mesh = Arc::new(Mesh::new_rubik_piece(
                        device,
                        faced_top,
                        faced_bottom,
                        faced_left,
                        faced_right,
                        faced_front,
                        faced_back,
                    ));
                    let mut cube = new_entity(rubik_mesh, shader.clone());
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
        let depth = rng.gen_range(1..self.span * 2) as f32 * size * 0.5;
        match self.current_move {
            Move::Top => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().z;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::Bottom => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().z;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::Left => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().x;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::Right => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().x;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::Front => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().y;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            Move::Back => {
                for piece in self.static_root.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().y;
                    layer < depth
                }) {
                    self.moving_pivot.add_child(piece.clone());
                }
            }
            _ => {}
        };
        let rotate_amount = PI * 0.5 * rng.gen_range(1..=3) as f32;
        match rng.gen_range(0..28) {
            0 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(BackIn)),
            1 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(BackInOut)),
            2 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(BackOut)),
            3 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(BounceIn)),
            4 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(BounceInOut)),
            5 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(BounceOut)),
            6 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(CircIn)),
            7 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(CircInOut)),
            8 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(CircOut)),
            9 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(CubicIn)),
            10 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(CubicInOut)),
            11 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(CubicOut)),
            12 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(ElasticIn)),
            13 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(ElasticInOut)),
            14 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(ElasticOut)),
            15 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(ExpoIn)),
            16 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(ExpoInOut)),
            17 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(ExpoOut)),
            18 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(QuadIn)),
            19 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(QuadInOut)),
            20 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(QuadOut)),
            21 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(QuintIn)),
            22 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(QuintInOut)),
            23 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(QuintOut)),
            24 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(SineIn)),
            25 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(SineInOut)),
            26 => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(SineOut)),
            _ => self.tween = Tweener::new(0.0, rotate_amount, 2.0, Box::new(Linear)),
        };
    }
    pub fn finish_move(&mut self) {
        let mat = self.moving_pivot.calculate_transform();
        for mut piece in self.moving_pivot.extract_child() {
            let mat = mat * piece.calculate_transform();
            let (_scale, rotation, translation) = mat.to_scale_rotation_translation();
            piece.translate(translation.x, translation.y, translation.z);
            piece.rotate_quat(rotation);
            self.static_root.add_child(piece);
        }
        self.moving_pivot.rotate(0.0, 0.0, 0.0);
        self.start_move_random();
    }
    pub fn update(&mut self, delta_time: f32) {
        let alpha = self.tween.move_by(delta_time);
        match self.current_move {
            Move::Top => {
                self.moving_pivot.rotate_z(alpha);
            }
            Move::Bottom => {
                self.moving_pivot.rotate_z(alpha);
            }
            Move::Left => {
                self.moving_pivot.rotate_x(alpha);
            }
            Move::Right => {
                self.moving_pivot.rotate_x(alpha);
            }
            Move::Front => {
                self.moving_pivot.rotate_y(alpha);
            }
            Move::Back => {
                self.moving_pivot.rotate_y(alpha);
            }
            _ => {}
        };
        if self.tween.is_finished() {
            self.finish_move();
        }
    }
}
