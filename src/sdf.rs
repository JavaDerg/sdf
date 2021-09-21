use nalgebra::Vector3;

pub trait Sdf {
    fn distance(&self, pt: &Vector3<f32>) -> f32;
    fn id(&self, pt: &Vector3<f32>) -> u32;
}

// impl<T: Sdf, R: AsRef<T>> Sdf for R {
//     fn distance(&self, pt: &Vector3<f32>) -> f32 {
//         self.distance(pt)
//     }
// }

pub struct Sphere(pub f32, pub Vector3<f32>, pub u32);

impl Sdf for Sphere {
    fn distance(&self, pt: &Vector3<f32>) -> f32 {
        (pt + &self.1).norm() - self.0
    }

    fn id(&self, _: &Vector3<f32>) -> u32 {
        self.2
    }
}

pub struct Union<S1: Sdf, S2: Sdf>(pub S1, pub S2);
pub struct Overlap<S1: Sdf, S2: Sdf>(pub S1, pub S2);
pub struct Cut<S1: Sdf, S2: Sdf>(pub S1, pub S2);

impl<S1: Sdf, S2: Sdf> Sdf for Union<S1, S2> {
    fn distance(&self, pt: &Vector3<f32>) -> f32 {
        self.0.distance(pt).min(self.1.distance(pt))
    }

    fn id(&self, pt: &Vector3<f32>) -> u32 {
        match self.0.distance(pt) < self.1.distance(pt) {
            true => self.0.id(pt),
            false => self.1.id(pt),
        }
    }
}

impl<S1: Sdf, S2: Sdf> Sdf for Overlap<S1, S2> {
    fn distance(&self, pt: &Vector3<f32>) -> f32 {
        self.0.distance(pt).max(self.1.distance(pt))
    }

    fn id(&self, pt: &Vector3<f32>) -> u32 {
        match self.0.distance(pt) > self.1.distance(pt) {
            true => self.0.id(pt),
            false => self.1.id(pt),
        }
    }
}

impl<S1: Sdf, S2: Sdf> Sdf for Cut<S1, S2> {
    fn distance(&self, pt: &Vector3<f32>) -> f32 {
        self.0.distance(pt).max(-self.1.distance(pt))
    }
    fn id(&self, pt: &Vector3<f32>) -> u32 {
        match self.0.distance(pt) > -self.1.distance(pt) {
            true => self.0.id(pt),
            false => self.1.id(pt),
        }
    }
}
