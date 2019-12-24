use std::f32::consts::PI;

use crate::ray::Ray;
use crate::vec::*;

pub struct Camera {
	origin: Vec3,
	lower_left_corner: Vec3,
	horizontal: Vec3,
	vertical: Vec3,
	u: Vec3,
	v: Vec3,
	lens_radius: f32,
}

impl Camera {
	pub fn new(
		look_from: Vec3,
		look_at: Vec3,
		up: Vec3,
		vertical_fov: f32,
		aspect: f32,
		aperture: f32,
		focus_distance: f32,
	) -> Camera {
		let theta = vertical_fov * PI / 180.0;
		let half_height = (theta / 2.0).tan();
		let half_width = aspect * half_height;
		let w = (look_from - look_at).normalize();
		let u = up.cross(&w).normalize();
		let v = w.cross(&u);
		Camera {
			origin: look_from,
			lower_left_corner: look_from
				- half_width * focus_distance * u
				- half_height * focus_distance * v
				- focus_distance * w,
			horizontal: 2.0 * half_width * focus_distance * u,
			vertical: 2.0 * half_height * focus_distance * v,
			u,
			v,
			lens_radius: aperture / 2.0,
		}
	}

	pub fn get_ray(&self, s: f32, t: f32) -> Ray {
		let rd = self.lens_radius * random_in_unit_disk();
		let offset = self.u * rd.x + self.v * rd.y;
		Ray {
			origin: self.origin + offset,
			direction: Unit::new_normalize(
				self.lower_left_corner + s * self.horizontal + t * self.vertical
					- self.origin - offset,
			),
		}
	}
}
