use crate::geometry::Geometry;
use crate::shader::Shader;
pub struct NodeVisual {
	pub geometry: Geometry,
	pub shader: Shader,
}
#[derive(Default)]
pub struct Node {
	pub visual: Option<NodeVisual>,
	pub children: Vec<Node>,
}

impl Node {
	pub fn new_empty() -> Self {
		Self {
			visual: None,
			children: Vec::new(),
		}
	}
	pub fn new(geometry: Geometry, shader: Shader) -> Self {
		Self {
			visual: Some(
				NodeVisual { geometry, shader }
			),
			children: Vec::new(),
		}
	}
	pub fn add_child(&mut self, node: Node) {
		self.children.push(node);
	}
}