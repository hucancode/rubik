use glam::{Mat4, Vec3};
use std::f32::consts;

pub struct Camera {
    pub distance: f32,
    pub azimuth: f32,  // Horizontal angle
    pub elevation: f32, // Vertical angle
    pub target: Vec3,
    pub fov: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            distance: 30.0,
            azimuth: 0.25,
            elevation: 0.5,
            target: Vec3::ZERO,
            fov: consts::FRAC_PI_4,
        }
    }
    
    pub fn orbit(&mut self, delta_azimuth: f32, delta_elevation: f32) {
        self.azimuth += delta_azimuth;
        self.elevation = (self.elevation + delta_elevation).clamp(-1.5, 1.5);
    }
    
    pub fn zoom(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(10.0, 100.0);
    }
    
    pub fn get_eye_position(&self) -> Vec3 {
        let x = self.distance * self.elevation.cos() * self.azimuth.sin();
        let y = self.distance * self.elevation.sin();
        let z = self.distance * self.elevation.cos() * self.azimuth.cos();
        Vec3::new(x, y, z) + self.target
    }
    
    pub fn make_vp_matrix(&self, aspect_ratio: f32) -> Mat4 {
        let projection = Mat4::perspective_rh(self.fov, aspect_ratio, 1.0, 1000.0);
        let view = Mat4::look_at_rh(self.get_eye_position(), self.target, Vec3::Y);
        projection * view
    }
    
    // Legacy function for compatibility
    pub fn make_vp_matrix_static(aspect_ratio: f32, distance: f32) -> Mat4 {
        let projection = Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 1000.0);
        let view = Mat4::look_at_rh(Vec3::new(1.0, -2.0, 1.0) * distance, Vec3::ZERO, Vec3::Z);
        projection * view
    }
}