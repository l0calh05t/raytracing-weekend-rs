use std::sync::Arc;

use crate::hittable::Hittable;
use crate::materials::*;
use crate::sphere::Sphere;
use crate::vec::*;

pub fn random_scene() -> Vec<Box<dyn Hittable + Sync>> {
	let mut list: Vec<Box<dyn Hittable + Sync>> = Vec::new();
	let ground_sphere_radius = 1000.0;
	list.push(Box::new(Sphere {
		center: Vec3::new(0.0, -ground_sphere_radius, 0.0),
		radius: ground_sphere_radius,
		material: Arc::new(Lambertian {
			albedo: Vec3::new(0.5, 0.5, 0.5),
		}),
	}));

	let to_ground = |center: Vec3, radius: f32| {
		(center - Vec3::new(0.0, -ground_sphere_radius, 0.0)).normalize()
			* (ground_sphere_radius + radius)
			+ Vec3::new(0.0, -ground_sphere_radius, 0.0)
	};

	let glass = Arc::new(Dielectric {
		refraction_index: 1.5,
	});
	let mut rng = rand::thread_rng();
	for a in -11..11 {
		for b in -11..11 {
			let choose_mat = rng.gen::<f32>();
			// modified centers to get a nicer distribution
			let center = Vec3::new(
				a as f32 + 0.2 + 0.6 * rng.gen::<f32>(),
				0.2,
				b as f32 + 0.2 + 0.6 * rng.gen::<f32>(),
			);
			// project center to ground sphere surface
			let center = to_ground(center, 0.2);
			// do not instantiate small spheres that intersect/touch any of large spheres
			if (center - to_ground(Vec3::new(0.0, 1.0, 0.0), 1.0)).norm() <= 1.2 {
				continue;
			};
			if (center - to_ground(Vec3::new(-4.0, 1.0, 0.0), 1.0)).norm() <= 1.2 {
				continue;
			};
			if (center - to_ground(Vec3::new(4.0, 1.0, 0.0), 1.0)).norm() <= 1.2 {
				continue;
			};
			list.push(Box::new(Sphere {
				center,
				radius: 0.2,
				material: if choose_mat < 0.8 {
					Arc::new(Lambertian {
						albedo: Vec3::new(
							rng.gen::<f32>() * rng.gen::<f32>(),
							rng.gen::<f32>() * rng.gen::<f32>(),
							rng.gen::<f32>() * rng.gen::<f32>(),
						),
					})
				} else if choose_mat < 0.95 {
					Arc::new(Metal {
						albedo: Vec3::new(
							0.5 * (1.0 + rng.gen::<f32>()),
							0.5 * (1.0 + rng.gen::<f32>()),
							0.5 * (1.0 + rng.gen::<f32>()),
						),
						fuzz: 0.5 * rng.gen::<f32>(),
					})
				} else {
					glass.clone()
				},
			}));
		}
	}

	list.push(Box::new(Sphere {
		center: to_ground(Vec3::new(0.0, 1.0, 0.0), 1.0),
		radius: 1.0,
		material: glass,
	}));
	list.push(Box::new(Sphere {
		center: to_ground(Vec3::new(-4.0, 1.0, 0.0), 1.0),
		radius: 1.0,
		material: Arc::new(Lambertian {
			albedo: Vec3::new(0.4, 0.2, 0.1),
		}),
	}));
	list.push(Box::new(Sphere {
		center: to_ground(Vec3::new(4.0, 1.0, 0.0), 1.0),
		radius: 1.0,
		material: Arc::new(Metal {
			albedo: Vec3::new(0.7, 0.6, 0.5),
			fuzz: 0.0,
		}),
	}));

	list
}
