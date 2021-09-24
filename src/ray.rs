use nalgebra::{Norm, Vector3};

#[derive(Clone)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>,
}
