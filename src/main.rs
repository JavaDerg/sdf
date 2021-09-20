use nalgebra::{Point4, Vector4, Point2, Vector2, Point3};
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use nalgebra as na;
use std::time::Instant;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;
const PIXELS: usize = WIDTH * HEIGHT;

struct World(Camera);
struct Camera {
    focal_len: f32,
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
            refs.push((row, 1.0 - (line as f32 + 0.5) / HEIGHT as f32 * 2.0));
            if last.is_empty() {
                break;
            }
        }

        let world = World(Camera {

        });
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
        *px = shader(world, Point2::from_slice(&[(w as f32 + 0.5) / WIDTH as f32 * 2.0 - 1.0, height]));
    }
}

fn dist_circ(pt: &Vector2<f32>, rad: f32) -> f32 {
    // match pt.norm() > rad {
    //     true => 1.0,
    //     false => 0.1,
    // }
    pt.norm() - rad
}

fn shader(world: &World, mut pt: Point2<f32>) -> Vector4<f32>  {
    pt.x *= WIDTH as f32 / HEIGHT as f32;

    if world.0.is_empty() {
        return Vector4::new(1.0, 1.0, 1.0, 1.0);
    }

    let mut val = {
        let c = world.0.get(0).unwrap();
        let op = &pt.coords + c.coords.xy();
        dist_circ(&op, c.z)
    };
    for c in world.0.iter().skip(1) {
        let op = &pt.coords + &c.coords.xy();
        val = val.max(-dist_circ(&op, c.z));
    }

    Vector4::new(val, val, val, 1.0)
}
