use crate::geometry::Mesh;
use crate::material::Shader;
use glam::{f32::Quat, EulerRot, Mat4, Vec3};
use std::sync::{Arc, Mutex};

pub enum Variant {
    Entity(Arc<Mesh>, Arc<Shader>),
    Light(wgpu::Color, f32),
    Group,
}

pub type NodeRef = Arc<Mutex<NodeData>>;

pub struct NodeData {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub variant: Variant,
    pub children: Vec<NodeRef>,
    pub parent: Option<NodeRef>,
}

impl Default for NodeData {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            variant: Variant::Group,
            children: Vec::new(),
            parent: None,
        }
    }
}

pub fn new_group() -> NodeRef {
    Arc::new(Mutex::new(NodeData::default()))
}

pub fn new_light(color: wgpu::Color, radius: f32) -> NodeRef {
    Arc::new(Mutex::new(NodeData {
        variant: Variant::Light(color, radius),
        ..Default::default()
    }))
}

pub fn new_entity(geometry: Arc<Mesh>, shader: Arc<Shader>) -> NodeRef {
    Arc::new(Mutex::new(NodeData {
        variant: Variant::Entity(geometry, shader),
        ..Default::default()
    }))
}

pub trait Node {
    fn get_translation(&self) -> Vec3;
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
    fn rotate_quat(&mut self, q: Quat);
    fn rotate(&mut self, x: f32, y: f32, z: f32);
    fn rotate_x(&mut self, x: f32);
    fn rotate_y(&mut self, y: f32);
    fn rotate_z(&mut self, z: f32);
    fn calculate_transform(&self) -> Mat4;
    fn add_child(&mut self, node: NodeRef);
    fn extract_child_if<F>(&mut self, filter: F) -> Vec<NodeRef>
    where
        F: Fn(&NodeRef) -> bool;
    fn extract_child(&mut self) -> Vec<NodeRef> {
        self.extract_child_if(|_| true)
    }
}
impl Node for NodeData {
    fn get_translation(&self) -> Vec3 {
        self.translation
    }
    fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.translation = Vec3::new(x, y, z);
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
        self.scale = Vec3::new(x, y, z);
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
    fn rotate_quat(&mut self, q: Quat) {
        self.rotation = q;
    }
    fn rotate(&mut self, x: f32, y: f32, z: f32) {
        self.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z);
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
    fn add_child(&mut self, child: NodeRef) {
        self.children.push(child);
    }
    fn extract_child_if<F>(&mut self, filter: F) -> Vec<NodeRef>
    where
        F: Fn(&NodeRef) -> bool,
    {
        let mut ret = Vec::new();
        let mut i = 0;
        while i < self.children.len() {
            if filter(&self.children[i]) {
                ret.push(self.children.swap_remove(i));
            } else {
                i += 1;
            }
        }
        ret
    }
    fn extract_child(&mut self) -> Vec<NodeRef> {
        let ret = self.children.clone();
        self.children = Vec::new();
        ret
    }
}
impl Node for NodeRef {
    fn get_translation(&self) -> Vec3 {
        if let Ok(node) = self.lock() {
            node.translation
        } else {
            eprintln!("Something went wrong!");
            Vec3::ZERO
        }
    }
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
    fn rotate_quat(&mut self, q: Quat) {
        if let Ok(mut node) = self.lock() {
            node.rotate_quat(q);
        }
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
    fn add_child(&mut self, child: NodeRef) {
        if let Ok(mut node) = self.lock() {
            if let Ok(mut child_mtx) = child.clone().lock() {
                node.children.push(child);
                child_mtx.parent = Some(self.clone());
            }
        }
    }
    fn extract_child_if<F>(&mut self, filter: F) -> Vec<NodeRef>
    where
        F: Fn(&NodeRef) -> bool,
    {
        if let Ok(mut node) = self.lock() {
            node.extract_child_if(filter)
        } else {
            Vec::new()
        }
    }
    fn extract_child(&mut self) -> Vec<NodeRef> {
        if let Ok(mut node) = self.lock() {
            node.extract_child()
        } else {
            Vec::new()
        }
    }
}
