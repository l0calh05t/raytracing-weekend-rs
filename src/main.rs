use nd::parallel::prelude::*;
use nd::prelude::*;
use ndarray as nd;

use std::time::Instant;

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

fn display_into_io<E: core::fmt::Display>(error: E) -> std::io::Error {
	std::io::Error::new(std::io::ErrorKind::Other, format!("{}", error))
}

#[cfg(feature = "use_oidn")]
fn debug_into_io<E: core::fmt::Debug>(error: E) -> std::io::Error {
	std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", error))
}

fn main() -> std::io::Result<()> {
	let mut instants = Vec::new();
	instants.reserve(6);
	instants.push((Instant::now(), ""));

	let mut args = std::env::args().skip(1);
	let nx = if let Some(s) = args.next() {
		s.parse().map_err(display_into_io)?
	} else {
		640
	};
	let ny = if let Some(s) = args.next() {
		s.parse().map_err(display_into_io)?
	} else {
		320
	};
	let ns = if let Some(s) = args.next() {
		s.parse().map_err(display_into_io)?
	} else if cfg!(feature = "use_oidn") {
		1
	} else {
		128
	};

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

	instants.push((Instant::now(), "initialization"));
	// use rayon to parallelize tracing over scan lines (better schemes possible, this was an easy, quick way)
	let mut img = Array::uninit((ny, nx));
	img.axis_iter_mut(Axis(0))
		.into_par_iter()
		.enumerate()
		.for_each(|(j, mut line)| {
			let mut rng = rand::thread_rng();
			line.iter_mut().enumerate().for_each(|(i, pixel)| {
				pixel.write(
					(0..ns)
						.map(|_| {
							let u = (i as f32 + rng.gen::<f32>()) / nx as f32;
							let v = (j as f32 + rng.gen::<f32>()) / ny as f32;
							let ray = camera.get_ray(u, v);
							color(&ray, &world, 0)
						})
						.sum::<Vec3>() / ns as f32,
				);
			})
		});
	let img = unsafe { img.assume_init() };

	instants.push((Instant::now(), "render"));

	#[cfg(feature = "use_oidn")]
	let img = {
		write_bmp("output_pre_oidn.bmp", img.view())?;
		instants.push((Instant::now(), "write (unfiltered)"));

		let in_slice_3 = img.view().to_slice().unwrap();

		// sanity checks to ensure alignment, size and element order of Vec3 match expectations on
		// f32 order in oidn, as the following cast is quite unsafe
		assert_eq!(
			std::mem::size_of_val(&in_slice_3[0]),
			3 * std::mem::size_of::<f32>()
		);
		assert_eq!(
			std::mem::align_of_val(&in_slice_3[0]),
			std::mem::align_of::<f32>()
		);
		assert_eq!(
			&in_slice_3[0].z as *const f32 as usize - &in_slice_3[0].x as *const f32 as usize,
			2 * std::mem::size_of::<f32>()
		);

		let in_slice = unsafe {
			core::slice::from_raw_parts(
				&in_slice_3[0] as *const _ as *const f32,
				3 * in_slice_3.len(),
			)
		};

		let mut out_img = Array::zeros((ny, nx));

		{
			let mut out_view = out_img.view_mut();
			let out_slice_3 = out_view.as_slice_mut().unwrap();
			let out_slice = unsafe {
				core::slice::from_raw_parts_mut(
					&mut out_slice_3[0] as *mut _ as *mut f32,
					3 * out_slice_3.len(),
				)
			};

			let device = oidn::Device::new();
			let mut filter = oidn::RayTracing::new(&device);
			filter.set_img_dims(nx, ny);

			filter.execute(in_slice, out_slice).map_err(debug_into_io)?;
			device.get_error().map_err(debug_into_io)?;
		}

		instants.push((Instant::now(), "denoise"));

		out_img
	};

	// write a binary bitmap instead of ascii ppm, because text formats are never the right choice
	write_bmp("output.bmp", img.view())?;

	instants.push((Instant::now(), "write"));

	instants
		.iter()
		.zip(instants.iter().skip(1))
		.for_each(|((t0, _), (t1, s))| println!("{:?} {}", t1.duration_since(*t0), s));

	Ok(())
}
