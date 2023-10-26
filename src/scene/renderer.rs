use crate::scene::Camera;
use crate::scene::Node;
pub struct Renderer<'a> {
	pub camera: Camera,
	pub root: Node<'a>,
}

impl<'a> Renderer<'a> {
	pub fn new() -> Self {
		Self {
			camera: Camera::new(),
			root: Node::new(),
		}
	}
}