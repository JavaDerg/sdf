use crate::{Color, Vec3f};
use nalgebra::{Vector2, Vector3, Vector4};
use std::sync::Arc;

pub type Uv = Vector2<f32>;

#[derive(Clone)]
pub struct Sdf {
    sdf: Arc<dyn Fn(&Vec3f) -> f32 + Send + Sync>,
    shader: Arc<dyn Fn(&Vec3f) -> Color + Send + Sync>,
    uv: Arc<dyn Fn(&Vec3f) -> Uv + Send + Sync>,
}

impl Sdf {
    pub fn compute_dist(&self, pt: &Vec3f) -> f32 {
        (self.sdf)(pt)
    }
    pub fn shade(&self, pt: &Vec3f) -> Color {
        (self.shader)(pt)
    }
}

// impl<T: Sdf, R: AsRef<T>> Sdf for R {
//     fn distance(&self, pt: &Vector3<f32>) -> f32 {
//         self.distance(pt)
//     }
// }

pub fn color_sphere(radius: f32, color: Color) -> Sdf {
    Sdf {
        sdf: Arc::new(move |pt| pt.norm() - radius),
        shader: Arc::new(move |_| color.clone()),
        uv: Arc::new(|pt| {
            let opt = pt.normalize();
            let v = 1.0 - (opt.dot(&Vector3::new(0.0, 1.0, 0.0)) + 1.0) / 2.0;
            let mut u = 0.5 - (opt.xz().normalize().dot(&Vector2::new(0.0, 1.0)) + 1.0) / 4.0;
            if opt.dot(&Vector3::new(1.0, 0.0, 0.0)).is_sign_positive() {
                u = 1.0 - u;
            }
            Vector2::new(u, v)
        }),
    }
}

// pub struct Sphere {
//     pub shader: Arc<dyn Fn(&Self, &Vec3f) -> Color + Send + Sync>,
//     pub inner: SphereInner,
// }
//
// pub struct SphereInner {
//     pub radius: f32,
//     pub location: Vector3<f32>,
// }

/*
impl Sdf for Sphere {
    fn distance(&self, pt: &Vector3<f32>) -> f32 {
        (pt + &self.inner.location).norm() - self.inner.radius
    }

    fn uv(&self, pt: &Vector3<f32>) -> Vector2<f32> {
        let opt = (pt + self.offset()).normalize();
        let v = 1.0 - (opt.dot(&Vector3::new(0.0, 1.0, 0.0)) + 1.0) / 2.0;
        let mut u = 0.5 - (opt.xz().normalize().dot(&Vector2::new(0.0, 1.0)) + 1.0) / 4.0;
        if opt.dot(&Vector3::new(1.0, 0.0, 0.0)).is_sign_positive() {
            u = 1.0 - u;
        }
        Vector2::new(u, v)
    }

    fn offset(&self) -> &Vector3<f32> {
        &self.inner.location
    }

    fn shade(&self, pt: &Vector3<f32>) -> Color {
        (*self.shader)(self, pt)
    }
}
*/
// pub struct Union<S1: Sdf, S2: Sdf>(pub S1, pub S2);
// pub struct Overlap<S1: Sdf, S2: Sdf>(pub S1, pub S2);
// pub struct Cut<S1: Sdf, S2: Sdf>(pub S1, pub S2);
//
// impl<S1: Sdf, S2: Sdf> Sdf for Union<S1, S2> {
//     fn distance(&self, pt: &Vector3<f32>) -> f32 {
//         self.0.distance(pt).min(self.1.distance(pt))
//     }
//
//     fn id(&self, pt: &Vector3<f32>) -> u32 {
//         match self.0.distance(pt) < self.1.distance(pt) {
//             true => self.0.id(pt),
//             false => self.1.id(pt),
//         }
//     }
// }
//
// impl<S1: Sdf, S2: Sdf> Sdf for Overlap<S1, S2> {
//     fn distance(&self, pt: &Vector3<f32>) -> f32 {
//         self.0.distance(pt).max(self.1.distance(pt))
//     }
//
//     fn id(&self, pt: &Vector3<f32>) -> u32 {
//         match self.0.distance(pt) > self.1.distance(pt) {
//             true => self.0.id(pt),
//             false => self.1.id(pt),
//         }
//     }
// }
//
// impl<S1: Sdf, S2: Sdf> Sdf for Cut<S1, S2> {
//     fn distance(&self, pt: &Vector3<f32>) -> f32 {
//         self.0.distance(pt).max(-self.1.distance(pt))
//     }
//     fn id(&self, pt: &Vector3<f32>) -> u32 {
//         match self.0.distance(pt) > -self.1.distance(pt) {
//             true => self.0.id(pt),
//             false => self.1.id(pt),
//         }
//     }
// }
