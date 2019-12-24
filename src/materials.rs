use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::vec::*;

pub struct Scatter {
	pub attenuation: Vec3,
	pub scattered: Ray,
}

pub trait Material {
	fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<Scatter>;
}

pub struct Lambertian {
	pub albedo: Vec3,
}

impl Material for Lambertian {
	fn scatter(&self, _ray: &Ray, record: &HitRecord) -> Option<Scatter> {
		let target = record.p + record.n.as_ref() + random_in_unit_sphere();
		Some(Scatter {
			attenuation: self.albedo,
			scattered: Ray {
				origin: record.p,
				direction: Unit::new_normalize(target - record.p),
			},
		})
	}
}

pub struct Metal {
	pub albedo: Vec3,
	pub fuzz: f32,
}

fn reflect(v: Unit<Vec3>, n: Unit<Vec3>) -> Vec3 {
	v.as_ref() - 2.0 * v.dot(&n) * n.as_ref()
}

impl Material for Metal {
	fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<Scatter> {
		let reflected = reflect(ray.direction, record.n);
		let returned = Scatter {
			attenuation: self.albedo,
			scattered: Ray {
				origin: record.p,
				direction: Unit::new_normalize(reflected + self.fuzz * random_in_unit_sphere()),
			},
		};
		if returned.scattered.direction.dot(&record.n) <= 0.0 {
			return None;
		};
		Some(returned)
	}
}

fn refract(v: Unit<Vec3>, n: Unit<Vec3>, ni_over_nt: f32) -> Option<Vec3> {
	let dt = v.dot(&n);
	let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);
	if discriminant <= 0.0 {
		return None;
	};
	Some(ni_over_nt * (v.as_ref() - n.as_ref() * dt) - n.as_ref() * discriminant.sqrt())
}

fn schlick(cosine: f32, refraction_index: f32) -> f32 {
	let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
	r0 *= r0;
	r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
}

pub struct Dielectric {
	pub refraction_index: f32,
}

impl Material for Dielectric {
	fn scatter(&self, ray: &Ray, record: &HitRecord) -> Option<Scatter> {
		let (outward_normal, ni_over_nt) = if ray.direction.dot(record.n.as_ref()) > 0.0 {
			(-record.n, self.refraction_index)
		} else {
			(record.n, 1.0 / self.refraction_index)
		};

		let direction = if let Some(refracted) = refract(ray.direction, outward_normal, ni_over_nt)
		{
			// no reasoning given for multiplication by ref_idx in first branch,
			// so let's just not do that and reduce the amount of code in the branch
			let cosine = -ray.direction.dot(&outward_normal);
			if rand::thread_rng().gen::<f32>() > schlick(cosine, self.refraction_index) {
				refracted
			} else {
				reflect(ray.direction, record.n)
			}
		} else {
			reflect(ray.direction, record.n)
		};

		Some(Scatter {
			attenuation: Vec3::new(1.0, 1.0, 1.0),
			scattered: Ray {
				origin: record.p,
				direction: Unit::new_normalize(direction),
			},
		})
	}
}
