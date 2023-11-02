use crate::geometry::Geometry;
use crate::shader::Shader;
use glam::{f32::Quat, Mat4, Vec3};
use std::sync::{Arc, Mutex};

pub struct NodeVisual {
    pub geometry: Arc<Geometry>,
    pub shader: Arc<Shader>,
}

pub struct NodeTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub struct Node {
    pub transform: Arc<Mutex<NodeTransform>>,
    pub visual: Option<NodeVisual>,
    pub children: Vec<Arc<Node>>,
}

impl NodeTransform {
    pub fn calculate(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Node {
    pub fn new_empty() -> Self {
        let transform = NodeTransform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        Self {
            transform: Arc::new(Mutex::new(transform)),
            visual: None,
            children: Vec::new(),
        }
    }
    pub fn new(geometry: Arc<Geometry>, shader: Arc<Shader>) -> Self {
        let transform = NodeTransform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        Self {
            transform: Arc::new(Mutex::new(transform)),
            visual: Some(NodeVisual { geometry, shader }),
            children: Vec::new(),
        }
    }
    pub fn add_child(&mut self, node: Arc<Node>) {
        self.children.push(node);
    }
}
