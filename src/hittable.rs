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

#[derive(Clone, Copy)]
pub struct Aabb {
	pub min: Vec3,
	pub max: Vec3,
}

impl Aabb {
	pub fn empty() -> Self {
		let pos_inf = std::f32::INFINITY;
		let neg_inf = std::f32::NEG_INFINITY;
		Self {
			min: Vec3::new(pos_inf, pos_inf, pos_inf),
			max: Vec3::new(neg_inf, neg_inf, neg_inf),
		}
	}

	pub fn merged(&self, other: &Self) -> Self {
		Self {
			min: Vec3::new(
				self.min.x.min(other.min.x),
				self.min.y.min(other.min.y),
				self.min.z.min(other.min.z),
			),
			max: Vec3::new(
				self.max.x.max(other.max.x),
				self.max.y.max(other.max.y),
				self.max.z.max(other.max.z),
			),
		}
	}

	pub fn intersects(&self, ray: &Ray, mut t_min: f32, mut t_max: f32) -> bool {
		let dir_inv = Vec3::new(
			1.0 / ray.direction.x,
			1.0 / ray.direction.y,
			1.0 / ray.direction.z,
		);

		for i in 0..3 {
			let t1 = (self.min[i] - ray.origin[i]) * dir_inv[i];
			let t2 = (self.max[i] - ray.origin[i]) * dir_inv[i];

			t_min = t_min.max(t1.min(t2).min(t_max));
			t_max = t_max.min(t1.max(t2).max(t_min));
		}

		t_max > t_min.max(0.0)
	}

	pub fn intersect(&self, other: &Self) -> Self {
		Self {
			min: Vec3::new(
				self.min.x.max(other.min.x),
				self.min.y.max(other.min.y),
				self.min.z.max(other.min.z),
			),
			max: Vec3::new(
				self.max.x.min(other.max.x),
				self.max.y.min(other.max.y),
				self.max.z.min(other.max.z),
			),
		}
	}

	pub fn volume(&self) -> f32 {
		if self.max.x <= self.min.x || self.max.y <= self.min.y || self.max.z <= self.min.z {
			0.0
		} else {
			let delta = self.max - self.min;
			delta.dot(&delta)
		}
	}
}

pub trait Hittable {
	fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
	fn aabb(&self) -> Aabb;
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
			if let Some(new_hit) = hittable.hit(&ray, t_min, closest_so_far) {
				hit = Some(new_hit);
			};
		}
		hit
	}

	fn aabb(&self) -> Aabb {
		self.iter().fold(Aabb::empty(), |b, h| b.merged(&h.aabb()))
	}
}

impl<H: Hittable + ?Sized> Hittable for Box<H> {
	fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
		self.as_ref().hit(&ray, t_min, t_max)
	}

	fn aabb(&self) -> Aabb {
		self.as_ref().aabb()
	}
}

pub struct Bvh<H> {
	hittables: Vec<H>,
	nodes: Vec<(Aabb, usize, usize, bool)>,
}

impl<H: Hittable> Bvh<H> {
	pub fn new(mut hittables: Vec<H>) -> Self {
		fn recurse<H: Hittable>(
			nodes: &mut Vec<(Aabb, usize, usize, bool)>,
			hittables: &mut [H],
			offset: usize,
			level: usize,
		) -> usize {
			// if number of hittables is below threshold, create a leaf node
			if hittables.len() <= 8 {
				let index = nodes.len();
				nodes.push((
					hittables
						.iter()
						.fold(Aabb::empty(), |b, h| b.merged(&h.aabb())),
					offset,
					hittables.len(),
					true,
				));
				return index;
			}

			// try to find a good split axis (determined by Aabb overlap)
			let mut axis = 0;
			let mut overlap = std::f32::INFINITY;
			for trial_axis in 0..3 {
				hittables.sort_unstable_by(|a, b| {
					let a = 0.5 * (a.aabb().min[trial_axis] + a.aabb().max[trial_axis]);
					let b = 0.5 * (b.aabb().min[trial_axis] + b.aabb().max[trial_axis]);
					a.partial_cmp(&b).unwrap()
				});
				let (left, right) = hittables.split_at(hittables.len() / 2);
				let left_aabb = left.iter().fold(Aabb::empty(), |b, h| b.merged(&h.aabb()));
				let right_aabb = right.iter().fold(Aabb::empty(), |b, h| b.merged(&h.aabb()));
				let trial_overlap = left_aabb.intersect(&right_aabb).volume();
				if trial_overlap < overlap {
					overlap = trial_overlap;
					axis = trial_axis;
				}
			}

			// sort according to selected split axis (don't repeat sort for last axis)
			if axis != 2 {
				hittables.sort_unstable_by(|a, b| {
					let a = 0.5 * (a.aabb().min[axis] + a.aabb().max[axis]);
					let b = 0.5 * (b.aabb().min[axis] + b.aabb().max[axis]);
					a.partial_cmp(&b).unwrap()
				});
			}

			// split and recurse to build left / right subtrees
			let (left, right) = hittables.split_at_mut(hittables.len() / 2);

			let left_index = recurse(nodes, left, offset, level + 1);
			let right_index = recurse(nodes, right, offset + left.len(), level + 1);

			let index = nodes.len();
			nodes.push((
				nodes[left_index].0.merged(&nodes[right_index].0),
				left_index,
				right_index,
				false,
			));
			index
		}

		let mut nodes = Vec::new();
		recurse(&mut nodes, hittables.as_mut_slice(), 0, 0);
		Self { hittables, nodes }
	}
}

impl<H: Hittable> Hittable for Bvh<H> {
	fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
		fn recurse<'a, H: 'a + Hittable>(
			bvh: &'a Bvh<H>,
			ray: &Ray,
			t_min: f32,
			t_max: f32,
			node: usize,
		) -> Option<HitRecord<'a>> {
			let node = &bvh.nodes[node];
			if !node.0.intersects(ray, t_min, t_max) {
				// ray does not intersect node Aabb
				None
			} else if node.3 {
				// leaf node
				let hittables = &bvh.hittables[node.1..(node.1 + node.2)];
				let mut hit: Option<HitRecord<'a>> = None;
				for hittable in hittables.iter() {
					let closest_so_far = if let Some(record) = hit {
						record.t
					} else {
						t_max
					};
					if let Some(new_hit) = hittable.hit(&ray, t_min, closest_so_far) {
						hit = Some(new_hit);
					};
				}
				hit
			} else {
				// interior node
				let hit = recurse(bvh, ray, t_min, t_max, node.1);
				let closest_so_far = if let Some(record) = hit {
					record.t
				} else {
					t_max
				};
				if let Some(new_hit) = recurse(bvh, ray, t_min, closest_so_far, node.2) {
					Some(new_hit)
				} else {
					hit
				}
			}
		}
		recurse(self, ray, t_min, t_max, self.nodes.len() - 1)
	}

	fn aabb(&self) -> Aabb {
		self.nodes[self.nodes.len() - 1].0
	}
}
