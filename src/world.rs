use crate::ray::Ray;
use crate::sdf::Sdf;
use crate::{Camera, Vec3f};
use hecs::Entity;
use nalgebra::Vector3;

pub struct World {
    pub ecs: hecs::World,
    pub camera: Camera,
}

pub struct Intersect {
    pub entity: Entity,
    pub sdf: Sdf,
    pub intersect: Vector3<f32>,
    pub local: Vector3<f32>,
    pub steps: u32,
    pub distance: f32,
}

enum IntersectMode {
    Not(Entity),
    None,
    One(Entity),
}

impl World {
    pub fn intersect<T: 'static + Send + Sync>(
        &self,
        ray: &Ray,
        threshold: f32,
        max: f32,
    ) -> Option<Intersect> {
        self.intersect_inner::<T>(ray, threshold, max, IntersectMode::None)
    }

    pub fn intersect_not<T: 'static + Send + Sync>(
        &self,
        not: Entity,
        ray: &Ray,
        threshold: f32,
        max: f32,
    ) -> Option<Intersect> {
        self.intersect_inner::<T>(ray, threshold, max, IntersectMode::Not(not))
    }

    pub fn intersect_one<T: 'static + Send + Sync>(
        &self,
        one: Entity,
        ray: &Ray,
        threshold: f32,
        max: f32,
    ) -> Option<Intersect> {
        self.intersect_inner::<T>(ray, threshold, max, IntersectMode::One(one))
    }

    fn intersect_inner<T: 'static + Send + Sync>(
        &self,
        ray: &Ray,
        threshold: f32,
        max: f32,
        mode: IntersectMode,
    ) -> Option<Intersect> {
        let mut query = self.ecs.query::<(&T, &Vec3f, &Sdf)>();
        let ts: Vec<(Entity, (&T, &Vec3f, &Sdf))> = query.iter().collect::<Vec<_>>();
        if ts.is_empty() {
            return None;
        }
        let mut step = ray.origin.clone();
        let mut steps = 1;
        loop {
            let (id, _, local, sdf, dist) = ts
                .iter()
                .filter(|(id, ..)| match &mode {
                    IntersectMode::Not(id2) => id2 != id,
                    IntersectMode::None => true,
                    IntersectMode::One(id2) => id2 == id,
                })
                .map(|(id, (t, p, s))| (id, t, &step - *p, s))
                .map(|(id, t, p, s)| (id, t, p, s, s.compute_dist(&p)))
                .min_by(|(_, _, _, _, d1), (_, _, _, _, d2)| d1.partial_cmp(d2).unwrap())?;
            if dist <= threshold {
                return Some(Intersect {
                    entity: *id,
                    sdf: (*sdf).clone(),
                    distance: (&ray.origin - &step).norm(),
                    intersect: step,
                    local,
                    steps,
                });
            } else if dist > max {
                return None;
            }
            step += ray.direction * dist;
            steps += 1;
        }
    }
}
