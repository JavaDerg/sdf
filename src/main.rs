mod light;
mod ray;
mod sdf;
mod world;

use crate::light::{Light, LightKind};
use crate::ray::Ray;
use crate::sdf::{color_sphere, Sdf};
use crate::world::{Intersect, World};
use mimalloc::MiMalloc;
use nalgebra as na;
use nalgebra::{Point2, Point3, Point4, Vector, Vector2, Vector3, Vector4};
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

const WIDTH: usize = 720; // 1920;
const HEIGHT: usize = 480; // 1080;
const PIXELS: usize = WIDTH * HEIGHT;
const ASPECT_RATIO: f32 = WIDTH as f32 / HEIGHT as f32;
const THRESHOLD: f32 = 0.00001;
const ILLUM_THRESHOLD: f32 = 0.000001;
const MAX_DIST: f32 = 100.0;
const SAMPLES: usize = 16;

type Color = Vector4<f32>;
type Vec3f = Vector3<f32>;

struct ColorWrapper(pub Color);

fn color(x: f32, y: f32, z: f32) -> Color {
    Color::new(x, y, z, 1.0)
}

pub struct Camera {
    origin: Vector3<f32>,
    focal_len: f32,
    sensor: Point2<f32>,
}

impl Camera {
    pub fn project(&self, v: Vector2<f32>) -> Vector3<f32> {
        Vector3::new(
            (v.x - 0.5) * self.sensor.x,
            -(v.y - 0.5) * self.sensor.y,
            -self.focal_len,
        )
        .normalize()
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

        let world = World {
            ecs: {
                let mut world = hecs::World::new();
                world.spawn((
                    (),
                    color_sphere(0.1, Color::new(1.0, 1.0, 1.0, 1.0)),
                    Vec3f::new(-1.0, 1.0, -9.5),
                ));
                world.spawn((
                    (),
                    color_sphere(1.0, Color::new(1.0, 1.0, 1.0, 1.0)),
                    Vec3f::new(0.0, 0.0, -10.0),
                ));
                world.spawn((
                    Light {
                        intensity: 5.0,
                        kind: LightKind::Point,
                    },
                    Vec3f::new(-3.0, 2.0, -9.0),
                    Color::new(1.0, 0.0, 0.0, 1.0),
                ));
                world.spawn((
                    Light {
                        intensity: 0.3,
                        kind: LightKind::Point,
                    },
                    Vec3f::new(1.0, 1.0, -9.0),
                    Color::new(0.0, 0.0, 1.0, 1.0),
                ));
                world
            },
            camera: Camera {
                origin: Vector3::new(0.0, 0.0, 0.0),
                focal_len: 50.0,
                sensor: Point2::from_slice(&[18.0 * ASPECT_RATIO, 18.0]),
            },
        };
        // let int = world.intersect::<()>(&Ray {
        //     origin: Vector3::new(0.0, 0.0, 0.0),
        //     direction: Vector3::new(0.0, 0.0, 1.0),
        // }, 0.001, 100.0).unwrap();
        //
        // println!("{}\n{}\n{}", int.steps, int.distance, int.intersect);

        // (
        //
        //     Box::new(
        //         // Union(
        //         //     Sphere(0.5, Vector3::new(0.75, 0.0, 10.0), 0),
        //         //     Union(
        //         //         Sphere(0.75, Vector3::new(0.0, 0.0, 10.0), 1),
        //         //         Sphere(1.0, Vector3::new(-1.0, 0.0, 10.0), 2)
        //         //     )
        //         // )
        //         Sphere {
        //             shader: Arc::new(|sph, pt| {
        //                 let uv = sph.uv(pt);
        //                 todo!()
        //             }),
        //             inner: SphereInner {
        //                 radius: 1.0,
        //                 location: Vector3::new(0.0, 0.0, 10.0),
        //             },
        //         },
        //     ),
        // );
        let start = Instant::now();
        refs.into_par_iter()
            .for_each(|(slice, h)| render_row(&world, slice, h));
        let stop = Instant::now();
        let diff = stop - start;
        println!(
            "Rendered frame; pixels={}({}x{}); threads={}; time={}ms",
            PIXELS,
            WIDTH,
            HEIGHT,
            rayon::current_num_threads(),
            diff.as_secs_f32() * 1000.0
        );
    };
    let mut file = File::create("out.ppm").unwrap();
    write!(&mut file, "P6 {} {} 255\n", WIDTH, HEIGHT).unwrap();
    for px in vec {
        let norm = px * 255.0;
        file.write_all(&[norm.x as u8, norm.y as u8, norm.z as u8])
            .unwrap();
    }
}

fn render_row(world: &World, pxs: &mut [Vector4<f32>], height: f32) {
    for (px, w) in pxs.iter_mut().zip(0..) {
        *px = Vector4::new(0.0, 0.0, 0.0, 1.0);
        // for p in &*AASQ {
        //     *px += shader(world, world.0.project(Vector2::new((w as f32 + p.x) / WIDTH as f32, (height + p.y) / HEIGHT as f32))) * p.z;
        // }
        for _ in 0..SAMPLES {
            *px += shader(
                world,
                world.camera.project(Vector2::new(
                    (w as f32 + fastrand::f32()) / WIDTH as f32,
                    (height + fastrand::f32()) / HEIGHT as f32,
                )),
            );
        }
        *px /= SAMPLES as f32;
    }
}

fn shader(world: &World, mut dir: Vector3<f32>) -> Vector4<f32> {
    let intersect = world.intersect::<()>(
        &Ray {
            origin: Vector3::new(0.0, 0.0, 0.0),
            direction: dir,
        },
        THRESHOLD,
        MAX_DIST,
    );

    let intersect = match intersect {
        Some(intersect) => intersect,
        None => return Vector4::new(0.0, 0.0, 0.0, 1.0),
    };

    let mut lightness = Color::new(0.0, 0.0, 0.0, 1.0);

    for (_id, (light, og_hit_pos, color)) in world.ecs.query::<(&Light, &Vec3f, &Color)>().iter() {
        let light: &Light = light;
        // TODO: Check for reflectance (tanget) and if the light is visiblefor the sphere it self
        match light.kind {
            LightKind::AllBright => {
                lightness = Color::new(light.intensity, light.intensity, light.intensity, 1.0);
                break;
            }
            // TODO: Check if visible
            LightKind::Point => {
                let emit = match world.intersect_not::<()>(
                    intersect.entity,
                    &Ray {
                        origin: intersect.intersect.clone(),
                        direction: og_hit_pos.clone(),
                    },
                    THRESHOLD,
                    MAX_DIST,
                ) {
                    Some(intersect) => og_hit_pos.dot(&intersect.intersect) < 0.0,
                    None => true,
                };
                if emit {
                    let falloff = 1.0 / (og_hit_pos - &intersect.intersect).norm_squared();
                    lightness += color * falloff * light.intensity;
                }
            }
            // TODO: Check if visible
            LightKind::Infinite(pt) => {
                let emit = world
                    .intersect_one::<()>(
                        intersect.entity,
                        &Ray {
                            origin: intersect.intersect.clone(),
                            direction: og_hit_pos.clone(),
                        },
                        THRESHOLD,
                        MAX_DIST,
                    )
                    .is_none();
                if emit {
                    lightness += color * light.intensity;
                }
            }
        }
    }

    let mut shade = intersect.sdf.shade(&intersect.local);

    // TODO: This is wrong, also adjust for alpha
    shade.x *= lightness.x;
    shade.y *= lightness.y;
    shade.z *= lightness.z;
    shade.w *= lightness.w;

    shade

    // let mut step = world.0.origin.clone();
    // let mut dist = world.1.distance(&step);
    //
    // // let mut steps = 0.0f32;
    // while dist.abs() > THRESHOLD {
    //     // steps += 1.0;
    //     step += dir * dist;
    //     dist = world.1.distance(&step);
    //     if dist > MAX_DIST {
    //         return Vector4::new(0.0, 0.0, 0.0, 1.0); // + background_shader(world, &dir);
    //     }
    // }
    //
    // let color = world.1.shade(&step);
    //
    // color

    // let fancy = steps / 10.0 / (&step - &world.0.origin).norm().max(THRESHOLD);

    // Vector4::new(fancy, fancy, fancy, 1.0)

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
