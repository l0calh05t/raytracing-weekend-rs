use nd::parallel::prelude::*;
use nd::prelude::*;
use ndarray as nd;

mod camera;
mod hittable;
mod materials;
mod ray;
mod scene;
mod sphere;
mod vec;
mod write_bmp;

use crate::camera::Camera;
use crate::hittable::Hittable;
use crate::ray::Ray;
use crate::scene::random_scene;
use crate::vec::*;
use crate::write_bmp::write_bmp;

fn color<H: Hittable>(ray: &Ray, world: &H, depth: i32) -> Vec3 {
	if let Some(record) = world.hit(ray, 1e-3, std::f32::MAX) {
		if depth < 50 {
			if let Some(scatter) = record.material.scatter(ray, &record) {
				let scattered = color(&scatter.scattered, world, depth + 1);
				return Vec3::new(
					scatter.attenuation.x * scattered.x,
					scatter.attenuation.y * scattered.y,
					scatter.attenuation.z * scattered.z,
				);
			}
		}
		return Vec3::zeros();
	}

	let unit_direction = ray.direction.normalize();
	let t = 0.5 * (unit_direction.y + 1.0);
	Vec3::new(1.0, 1.0, 1.0).lerp(&Vec3::new(0.5, 0.7, 1.0), t)
}

fn main() -> std::io::Result<()> {
	let nx = 640;
	let ny = 320;
	let ns = 128;

	let world = random_scene();

	let look_from = Vec3::new(13.0, 2.0, 3.0);
	let look_at = Vec3::new(0.0, 0.0, 0.0);
	let distance_to_focus = 10.0;
	let aperture = 0.1;
	let camera = Camera::new(
		look_from,
		look_at,
		Vec3::new(0.0, 1.0, 0.0),
		20.0,
		nx as f32 / ny as f32,
		aperture,
		distance_to_focus,
	);

	// use rayon to parallelize tracing over scan lines (better schemes possible, this was an easy, quick way)
	let mut img = unsafe { Array::uninitialized((ny, nx)) };
	img.axis_iter_mut(Axis(0))
		.into_par_iter()
		.enumerate()
		.for_each(|(j, mut line)| {
			let mut rng = rand::thread_rng();
			line.iter_mut().enumerate().for_each(|(i, pixel)| {
				*pixel = (0..ns)
					.map(|_| {
						let u = (i as f32 + rng.gen::<f32>()) / nx as f32;
						let v = (j as f32 + rng.gen::<f32>()) / ny as f32;
						let ray = camera.get_ray(u, v);
						color(&ray, &world, 0)
					})
					.sum::<Vec3>() / ns as f32;
			})
		});
	let img = img;

	// write a binary bitmap instead of ascii ppm, because text formats are never the right choice
	write_bmp("output.bmp", img.view())?;

	Ok(())
}
