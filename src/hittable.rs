use crate::materials::Material;
use crate::ray::Ray;
use crate::vec::{Unit, Vec3};

#[derive(Clone, Copy)]
pub struct HitRecord<'a> {
	pub t: f32,
	pub p: Vec3,
	pub n: Unit<Vec3>,
	pub material: &'a dyn Material,
}

pub trait Hittable {
	fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

impl<H: Hittable> Hittable for Vec<H> {
	fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
		let mut hit: Option<HitRecord> = None;
		for hittable in self.iter() {
			let closest_so_far = if let Some(record) = hit {
				record.t
			} else {
				t_max
			};
			if let Some(new_hit) = hittable.hit(ray, t_min, closest_so_far) {
				hit = Some(new_hit);
			};
		}
		hit
	}
}

impl<H: Hittable + ?Sized> Hittable for Box<H> {
	fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
		self.as_ref().hit(ray, t_min, t_max)
	}
}
