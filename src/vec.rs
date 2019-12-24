pub use na::Unit;
pub use nalgebra as na;
pub use rand::prelude::*;

pub type Vec2 = na::Vector2<f32>;
pub type Vec3 = na::Vector3<f32>;

pub fn random_in_unit_disk() -> Vec2 {
	let mut rng = rand::thread_rng();
	let mut p;
	while {
		p = 2.0 * Vec2::new(rng.gen(), rng.gen()) - Vec2::new(1.0, 1.0);
		p.norm_squared() >= 1.0
	} {}
	p
}

pub fn random_in_unit_sphere() -> Vec3 {
	let mut rng = rand::thread_rng();
	let mut p;
	while {
		p = 2.0 * Vec3::new(rng.gen(), rng.gen(), rng.gen()) - Vec3::new(1.0, 1.0, 1.0);
		p.norm_squared() >= 1.0
	} {}
	p
}
