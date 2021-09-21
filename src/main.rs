mod sdf;

use nalgebra::{Point4, Vector4, Point2, Vector2, Point3, Vector3};
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use nalgebra as na;
use std::time::Instant;
use crate::sdf::{Sdf, Sphere, Union, Cut, Overlap};

lazy_static::lazy_static! {
    static ref AASQ: Vec<Vector3<f32>> = {
        let mut vec = vec![];
        for i in 0..SAMPLES {
            let dst = (i as f32 / (SAMPLES as f32 - 1.0)).sqrt();
            let angle = 2.0 * std::f32::consts::PI * 0.618033 * i as f32;
            let x = dst * angle.cos();
            let y = dst * angle.sin();
            vec.push(Vector3::new(x, y, 1.0 / SAMPLES as f32));
        }
        vec
        // vec![
        //     Vector3::new(-1.0, 1.0, 0.003765), Vector3::new(-0.5, 1.0, 0.015019), Vector3::new(0.0, 1.0, 0.023792), Vector3::new(0.5, 1.0, 0.015019), Vector3::new(1.0, 1.0, 0.003765),
        //     Vector3::new(-1.0, 0.5, 0.015019), Vector3::new(-0.5, 0.5, 0.059912), Vector3::new(0.0, 0.5, 0.094907), Vector3::new(0.5, 0.5, 0.059912), Vector3::new(1.0, 0.5, 0.015019),
        //     Vector3::new(-1.0, 0.0, 0.023792), Vector3::new(-0.5, 0.0, 0.094907), Vector3::new(0.0, 0.0, 0.150342), Vector3::new(0.5, 0.0, 0.094907), Vector3::new(1.0, 0.0, 0.023792),
        //     Vector3::new(-1.0, -0.5, 0.015019), Vector3::new(-0.5, -0.5, 0.059912), Vector3::new(0.0, -0.5, 0.094907), Vector3::new(0.5, -0.5, 0.059912), Vector3::new(1.0, -0.5, 0.015019),
        //     Vector3::new(-1.0, -1.0, 0.003765), Vector3::new(-0.5, -1.0, 0.015019), Vector3::new(0.0, -1.0, 0.023792), Vector3::new(0.5, -1.0, 0.015019), Vector3::new(1.0, -1.0, 0.003765),
        // ]
    };
}

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const PIXELS: usize = WIDTH * HEIGHT;
const ASPECT_RATIO: f32 = WIDTH as f32 / HEIGHT as f32;
const THRESHOLD: f32 = 0.00001;
const MAX_DIST: f32 = 100.0;
const SAMPLES: usize = 16;

struct World(Camera, Box<dyn Sdf + 'static + Sync>);

struct Camera {
    origin: Vector3<f32>,
    focal_len: f32,
    sensor: Point2<f32>,
}

impl Camera {
    pub fn project(&self, v: Vector2<f32>) -> Vector3<f32> {
        Vector3::new((v.x - 0.5) * self.sensor.x, -(v.y - 0.5) * self.sensor.y, -self.focal_len).normalize()
    }
}

fn rf32() -> f32 {
    1.0 - fastrand::f32() * 2.0
}

fn main() {
    let mut vec = vec![Vector4::new(0.0f32, 0.0f32, 0.0f32, 0.0f32); PIXELS];
    {
        let mut refs = vec![];
        let mut last = &mut vec[..];
        for line in 0.. {
            let (row, other) = last.split_at_mut(WIDTH);
            last = other;
            refs.push((row, line as f32));
            if last.is_empty() {
                break;
            }
        }

        let world = World(Camera {
            origin: Vector3::new(0.0, 0.0, 0.0),
            focal_len: 50.0,
            sensor: Point2::from_slice(&[18.0*ASPECT_RATIO, 18.0])
        }, Box::new(
            Union(
                Sphere(0.5, Vector3::new(0.75, 0.0, 10.0), 0),
                Union(
                    Sphere(0.75, Vector3::new(0.0, 0.0, 10.0), 1),
                    Sphere(1.0, Vector3::new(-1.0, 0.0, 10.0), 2)
                )
            )
        ));
        let start = Instant::now();
        refs.into_par_iter().for_each(|(slice, h)| render_row(&world, slice, h));
        let stop = Instant::now();
        let diff = stop - start;
        println!("Rendered frame; pixels={}({}x{}); threads={}; time={}ms", PIXELS, WIDTH, HEIGHT, rayon::current_num_threads(), diff.as_secs_f32() * 1000.0);
    };
    let mut file = File::create("out.ppm").unwrap();
    write!(&mut file, "P6 {} {} 255\n", WIDTH, HEIGHT).unwrap();
    for px in vec {
        let norm = px * 255.0;
        file.write_all(&[norm.x as u8, norm.y as u8, norm.z as u8]).unwrap();
    }
}

fn render_row(world: &World, pxs: &mut [Vector4<f32>], height: f32) {
    for (px, w) in pxs.iter_mut().zip(0..) {
        *px = Vector4::new(0.0, 0.0, 0.0, 1.0);
        // for p in &*AASQ {
        //     *px += shader(world, world.0.project(Vector2::new((w as f32 + p.x) / WIDTH as f32, (height + p.y) / HEIGHT as f32))) * p.z;
        // }
        for _ in 0..SAMPLES {
            *px += shader(world, world.0.project(Vector2::new((w as f32 + fastrand::f32()) / WIDTH as f32, (height + fastrand::f32()) / HEIGHT as f32)));
        }
        *px /= SAMPLES as f32;
    }
}

fn id2color(id: u32) -> Vector4<f32> {
    match id {
        0 => Vector4::new(1.0, 0.0, 0.0, 1.0),
        1 => Vector4::new(0.0, 1.0, 0.0, 1.0),
        2 => Vector4::new(0.0, 0.0, 1.0, 1.0),
        _ => unreachable!()
    }
}

fn shader(world: &World, mut dir: Vector3<f32>) -> Vector4<f32> {
    let mut step = world.0.origin.clone();
    let mut dist = world.1.distance(&step);

    let mut color_sum = Vector4::new(0.0, 0.0, 0.0, 1.0);
    if dist < 0.01 {
        color_sum += id2color(world.1.id(&step));
    }
    let mut steps = 0.0f32;
    while dist.abs() > THRESHOLD {
        steps += 1.0;
        step += dir * dist;
        dist = world.1.distance(&step);
        if dist < 0.01 {
            color_sum += id2color(world.1.id(&step)) * (1.0 / dist);
        }
        if dist > MAX_DIST {
            let mut fancy = steps / (&step - &world.0.origin).norm().max(THRESHOLD);
            if fancy < 0.7 {
                fancy = 0.0;
            }
            color_sum.w = 0.0;
            color_sum.normalize_mut();
            color_sum *= fancy;
            color_sum.w = 1.0;
            return color_sum; // + background_shader(world, &dir);
        }
    }

    let fancy = steps / 10.0 / (&step - &world.0.origin).norm().max(THRESHOLD);
    color_sum.w = 0.0;
    color_sum.normalize_mut();
    color_sum *= fancy;
    color_sum.w = 1.0;
    color_sum
    // Vector4::new(fancy, fancy, fancy, 1.0)

    //

    // let mut val = {
    //     let c = world.0.get(0).unwrap();
    //     let op = &pt.coords + c.coords.xy();
    //     dist_circ(&op, c.z)
    // };
    // for c in world.0.iter().skip(1) {
    //     let op = &pt.coords + &c.coords.xy();
    //     val = val.max(-dist_circ(&op, c.z));
    // }

    // Vector4::new(val, val, val, 1.0)
}

fn background_shader(_world: &World, dir: &Vector3<f32>) -> Vector4<f32> {
    if dir.y < 0.0 {
        Vector4::new(0.1, 0.4, 0.1, 1.0)
    } else {
        Vector4::new(dir.y * 2.0, dir.y * 4.0, 1.0, 1.0)
    }
}
