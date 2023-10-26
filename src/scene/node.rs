use crate::geometry::Geometry;
use crate::shader::Shader;
pub struct NodeVisual<'a> {
	pub geometry: &'a Geometry,
	pub shader: &'a Shader,
}
pub struct Node<'a> {
	pub visual: Option<NodeVisual<'a>>,
	pub children: Vec<&'a Node<'a>>,
}

impl<'a> Node<'a> {
	pub fn new() -> Self {
		Self {
			visual: None,
			children: Vec::new(),
		}
	}
}