use std::sync::Arc;

use crate::hittable::{Aabb, HitRecord, Hittable};
use crate::materials::Material;
use crate::ray::Ray;
use crate::vec::*;

pub struct Sphere {
	pub center: Vec3,
	pub radius: f32,
	pub material: Arc<dyn Material + Send + Sync>,
}

impl Hittable for Sphere {
	fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
		let oc = ray.origin - self.center;
		let a = ray.direction.dot(&ray.direction);
		let b = 2.0 * oc.dot(&ray.direction);
		let c = oc.dot(&oc) - self.radius * self.radius;
		let discriminant = b * b - 4.0 * a * c;
		if discriminant < 0.0 {
			return None;
		};
		let mut t = (-b - discriminant.sqrt()) / (2.0 * a);
		if t < t_min || t > t_max {
			t = (-b + discriminant.sqrt()) / (2.0 * a);
			if t < t_min || t > t_max {
				return None;
			}
		};
		let hit_position = ray.point_at_parameter(t);
		Some(HitRecord {
			t,
			p: hit_position,
			n: Unit::new_unchecked((hit_position - self.center) / self.radius),
			material: &*self.material,
		})
	}

	fn aabb(&self) -> Aabb {
		let radius3 = Vec3::new(self.radius, self.radius, self.radius);
		Aabb {
			min: self.center - radius3,
			max: self.center + radius3,
		}
	}
}
