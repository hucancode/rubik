use glam::{Mat4, Vec3};
use std::f32::consts;
pub struct Camera {

}

impl Camera {
	pub fn new() -> Self {
		Self {

		}
	}
	pub fn make_vp_matrix(aspect_ratio: f32) -> Mat4 {
		let projection = Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
		let view = Mat4::look_at_rh(Vec3::new(1.5, -5.0, 3.0), Vec3::ZERO, Vec3::Z);
		projection * view
	}
}