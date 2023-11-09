use crate::geometry::Geometry;
use crate::shader::Shader;
use glam::{f32::Quat, Mat4, Vec3};
use std::sync::{Arc, Mutex};

pub enum Variant {
    Entity(Arc<Geometry>, Arc<Shader>),
    Light(wgpu::Color),
    Group,
}

pub struct Node {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub variant: Variant,
    pub children: Vec<Arc<Mutex<Node>>>,
    pub parent: Option<Arc<Mutex<Node>>>,
}

impl Node {
    pub fn new_group() -> Arc<Mutex<Self>> {
        let ret = Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            variant: Variant::Group,
            children: Vec::new(),
            parent: None,
        };
        Arc::new(Mutex::new(ret))
    }
    pub fn new_light(color: wgpu::Color) -> Arc<Mutex<Self>> {
        let ret = Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            variant: Variant::Light(color),
            children: Vec::new(),
            parent: None,
        };
        Arc::new(Mutex::new(ret))
    }
    pub fn new_entity(geometry: Arc<Geometry>, shader: Arc<Shader>) -> Arc<Mutex<Self>> {
        let ret = Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            variant: Variant::Entity(geometry, shader),
            children: Vec::new(),
            parent: None,
        };
        Arc::new(Mutex::new(ret))
    }
}

pub trait TreeNode {
    fn translate(&mut self, x: f32, y: f32, z: f32);
    fn translate_x(&mut self, x: f32);
    fn translate_y(&mut self, y: f32);
    fn translate_z(&mut self, z: f32);
    fn scale(&mut self, x: f32, y: f32, z: f32);
    fn scale_x(&mut self, x: f32);
    fn scale_y(&mut self, y: f32);
    fn scale_z(&mut self, z: f32);
    fn scale_uniform(&mut self, v: f32) {
        self.scale(v, v, v)
    }
    fn rotate(&mut self, x: f32, y: f32, z: f32);
    fn rotate_x(&mut self, x: f32);
    fn rotate_y(&mut self, y: f32);
    fn rotate_z(&mut self, z: f32);
    fn calculate_transform(&self) -> Mat4;
    fn add_child(&mut self, node: Arc<Mutex<Node>>);
}
impl TreeNode for Node {
    fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.translation = glam::Vec3::new(x, y, z);
    }
    fn translate_x(&mut self, x: f32) {
        self.translate(x, 0.0, 0.0)
    }
    fn translate_y(&mut self, y: f32) {
        self.translate(0.0, y, 0.0)
    }
    fn translate_z(&mut self, z: f32) {
        self.translate(0.0, 0.0, z)
    }
    fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = glam::Vec3::new(x, y, z);
    }
    fn scale_x(&mut self, x: f32) {
        self.scale(x, 0.0, 0.0)
    }
    fn scale_y(&mut self, y: f32) {
        self.scale(0.0, y, 0.0)
    }
    fn scale_z(&mut self, z: f32) {
        self.scale(0.0, 0.0, z)
    }
    fn scale_uniform(&mut self, v: f32) {
        self.scale(v, v, v)
    }
    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
    }
    fn rotate_x(&mut self, x: f32) {
        self.rotate(x, 0.0, 0.0);
    }
    fn rotate_y(&mut self, y: f32) {
        self.rotate(0.0, y, 0.0);
    }
    fn rotate_z(&mut self, z: f32) {
        self.rotate(0.0, 0.0, z);
    }
    fn calculate_transform(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
    fn add_child(&mut self, child: Arc<Mutex<Node>>) {
        self.children.push(child);
    }
}
impl TreeNode for Arc<Mutex<Node>> {
    fn translate(&mut self, x: f32, y: f32, z: f32) {
        if let Ok(mut node) = self.lock() {
            node.translate(x, y, z)
        }
    }
    fn translate_x(&mut self, x: f32) {
        self.translate(x, 0.0, 0.0)
    }
    fn translate_y(&mut self, y: f32) {
        self.translate(0.0, y, 0.0)
    }
    fn translate_z(&mut self, z: f32) {
        self.translate(0.0, 0.0, z)
    }
    fn scale(&mut self, x: f32, y: f32, z: f32) {
        if let Ok(mut node) = self.lock() {
            node.scale(x, y, z)
        }
    }
    fn scale_x(&mut self, x: f32) {
        self.scale(x, 0.0, 0.0)
    }
    fn scale_y(&mut self, y: f32) {
        self.scale(0.0, y, 0.0)
    }
    fn scale_z(&mut self, z: f32) {
        self.scale(0.0, 0.0, z)
    }
    fn scale_uniform(&mut self, v: f32) {
        self.scale(v, v, v)
    }
    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        if let Ok(mut node) = self.lock() {
            node.rotate(x, y, z)
        }
    }
    fn rotate_x(&mut self, x: f32) {
        self.rotate(x, 0.0, 0.0);
    }
    fn rotate_y(&mut self, y: f32) {
        self.rotate(0.0, y, 0.0);
    }
    fn rotate_z(&mut self, z: f32) {
        self.rotate(0.0, 0.0, z);
    }
    fn calculate_transform(&self) -> Mat4 {
        if let Ok(node) = self.lock() {
            Mat4::from_scale_rotation_translation(node.scale, node.rotation, node.translation)
        } else {
            Mat4::IDENTITY
        }
    }
    fn add_child(&mut self, child: Arc<Mutex<Node>>) {
        if let Ok(mut node) = self.lock() {
            if let Ok(mut child_mtx) = child.clone().lock() {
                node.children.push(child);
                child_mtx.parent = Some(self.clone());
            }
        }
    }
}
