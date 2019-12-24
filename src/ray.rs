use crate::vec::*;

#[derive(Clone, Copy)]
pub struct Ray {
	pub origin: Vec3,
	pub direction: Unit<Vec3>,
}

impl Ray {
	pub fn point_at_parameter(&self, t: f32) -> Vec3 {
		self.origin + t * self.direction.as_ref()
	}
}
