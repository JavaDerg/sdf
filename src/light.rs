use crate::{Color, Vec3f};
use nalgebra::Vector3;
use std::ops::{Div, Mul};

pub struct Light {
    pub intensity: f32,
    pub kind: LightKind,
}

#[non_exhaustive]
pub enum LightKind {
    AllBright,
    Point,
    Infinite(Vec3f),
}
