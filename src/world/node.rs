use crate::geometry::Geometry;
use crate::shader::Shader;
use glam::{f32::Quat, Mat4, Vec3};
use std::sync::{Arc, Mutex};

pub enum Variant {
    Entity(Arc<Geometry>, Arc<Shader>),
    Light(wgpu::Color),
    Group,
}

pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub struct Node {
    pub transform: Arc<Mutex<Transform>>,
    pub variant: Variant,
    pub children: Vec<Arc<Node>>,
}

impl Transform {
    pub fn calculate(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl Node {
    pub fn new_group() -> Self {
        let transform = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        Self {
            transform: Arc::new(Mutex::new(transform)),
            variant: Variant::Group,
            children: Vec::new(),
        }
    }
    pub fn new_light(color: wgpu::Color) -> Self {
        let transform = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        Self {
            transform: Arc::new(Mutex::new(transform)),
            variant: Variant::Light(color),
            children: Vec::new(),
        }
    }
    pub fn new_entity(geometry: Arc<Geometry>, shader: Arc<Shader>) -> Self {
        let transform = Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        Self {
            transform: Arc::new(Mutex::new(transform)),
            variant: Variant::Entity(geometry, shader),
            children: Vec::new(),
        }
    }
    pub fn add_child(&mut self, node: Arc<Node>) {
        self.children.push(node);
    }
}
