use crate::geometry::Mesh;
use crate::material::ShaderLit;
use crate::rubik::Move;
use crate::world::{new_entity, new_group, Node, NodeRef, Renderer};
use rand::Rng;
use std::f32::consts::PI;
use std::rc::Rc;
use tween::{
    BackIn, BackInOut, BackOut, BounceIn, BounceInOut, BounceOut, CircIn, CircInOut, CircOut,
    CubicIn, CubicInOut, CubicOut, ElasticIn, ElasticInOut, ElasticOut, ExpoIn, ExpoInOut, ExpoOut,
    Linear, QuadIn, QuadInOut, QuadOut, QuintIn, QuintInOut, QuintOut, SineIn, SineInOut, SineOut,
    Tween, Tweener,
};

const CUBE_SIZE: f32 = 2.0;
const CUBE_MARGIN: f32 = 0.15;

type GenericTween = Tweener<f32, f32, Box<dyn Tween<f32>>>;

pub struct Rubik {
    tween: GenericTween,
    current_move: Move,
    pub root: NodeRef,
    moving_pieces: NodeRef,
    static_pieces: NodeRef,
    span: usize,
}

impl Rubik {
    pub fn new() -> Self {
        let moving_cubes = new_group();
        let static_cubes = new_group();
        let mut root = new_group();
        root.add_child(moving_cubes.clone());
        root.add_child(static_cubes.clone());
        Self {
            tween: Tweener::new(0.0, PI * 2.0, 2.0, Box::new(Linear)),
            current_move: Move::None,
            root,
            moving_pieces: moving_cubes,
            static_pieces: static_cubes,
            span: 0,
        }
    }
    pub fn generate_pieces(&mut self, span: usize, renderer: &Renderer) {
        let shader = Rc::new(ShaderLit::new(renderer));
        let d = CUBE_SIZE + CUBE_MARGIN;
        self.span = span;
        let n = span as i32;
        for z in -n..=n {
            for y in -n..=n {
                for x in -n..=n {
                    let faced_top = z == n;
                    let faced_bottom = z == -n;
                    let faced_left = x == -n;
                    let faced_right = x == n;
                    let faced_front = y == n;
                    let faced_back = y == -n;
                    let visible = faced_top
                        || faced_bottom
                        || faced_left
                        || faced_left
                        || faced_right
                        || faced_front
                        || faced_back;
                    if !visible {
                        continue;
                    }
                    let rubik_mesh = Rc::new(Mesh::new_rubik_piece(
                        &renderer.device,
                        faced_top,
                        faced_bottom,
                        faced_left,
                        faced_right,
                        faced_front,
                        faced_back,
                    ));
                    let mut cube = new_entity(rubik_mesh, shader.clone());
                    self.static_pieces.add_child(cube.clone());
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
                for piece in self.static_pieces.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().z;
                    layer < depth
                }) {
                    self.moving_pieces.add_child(piece.clone());
                }
            }
            Move::Bottom => {
                for piece in self.static_pieces.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().z;
                    layer < depth
                }) {
                    self.moving_pieces.add_child(piece.clone());
                }
            }
            Move::Left => {
                for piece in self.static_pieces.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().x;
                    layer < depth
                }) {
                    self.moving_pieces.add_child(piece.clone());
                }
            }
            Move::Right => {
                for piece in self.static_pieces.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().x;
                    layer < depth
                }) {
                    self.moving_pieces.add_child(piece.clone());
                }
            }
            Move::Front => {
                for piece in self.static_pieces.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) - piece.get_translation().y;
                    layer < depth
                }) {
                    self.moving_pieces.add_child(piece.clone());
                }
            }
            Move::Back => {
                for piece in self.static_pieces.extract_child_if(|piece| {
                    let layer = size * (self.span as f32) + piece.get_translation().y;
                    layer < depth
                }) {
                    self.moving_pieces.add_child(piece.clone());
                }
            }
            _ => {}
        };
        let rotate_amount = PI * 0.5 * rng.gen_range(1..=3) as f32;
        let rotate_time = 0.5 + 0.1 * rng.gen_range(0..10) as f32;
        match rng.gen_range(0..28) {
            0 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(BackIn)),
            1 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(BackInOut)),
            2 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(BackOut)),
            3 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(BounceIn)),
            4 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(BounceInOut)),
            5 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(BounceOut)),
            6 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(CircIn)),
            7 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(CircInOut)),
            8 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(CircOut)),
            9 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(CubicIn)),
            10 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(CubicInOut)),
            11 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(CubicOut)),
            12 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(ElasticIn)),
            13 => {
                self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(ElasticInOut))
            }
            14 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(ElasticOut)),
            15 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(ExpoIn)),
            16 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(ExpoInOut)),
            17 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(ExpoOut)),
            18 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(QuadIn)),
            19 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(QuadInOut)),
            20 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(QuadOut)),
            21 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(QuintIn)),
            22 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(QuintInOut)),
            23 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(QuintOut)),
            24 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(SineIn)),
            25 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(SineInOut)),
            26 => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(SineOut)),
            _ => self.tween = Tweener::new(0.0, rotate_amount, rotate_time, Box::new(Linear)),
        };
    }
    pub fn finish_move(&mut self) {
        let mat = self.moving_pieces.calculate_transform();
        for mut piece in self.moving_pieces.extract_all_child() {
            let mat = mat * piece.calculate_transform();
            let (_scale, rotation, translation) = mat.to_scale_rotation_translation();
            piece.translate(translation.x, translation.y, translation.z);
            piece.rotate_quat(rotation);
            self.static_pieces.add_child(piece);
        }
        self.moving_pieces.rotate(0.0, 0.0, 0.0);
        self.start_move_random();
    }
    pub fn update(&mut self, delta_time: f32) {
        let alpha = self.tween.move_by(delta_time);
        match self.current_move {
            Move::Top => {
                self.moving_pieces.rotate_z(alpha);
            }
            Move::Bottom => {
                self.moving_pieces.rotate_z(alpha);
            }
            Move::Left => {
                self.moving_pieces.rotate_x(alpha);
            }
            Move::Right => {
                self.moving_pieces.rotate_x(alpha);
            }
            Move::Front => {
                self.moving_pieces.rotate_y(alpha);
            }
            Move::Back => {
                self.moving_pieces.rotate_y(alpha);
            }
            _ => {}
        };
        if self.tween.is_finished() {
            self.finish_move();
        }
    }
}
