use std::fs::File;
use std::io::BufWriter;
use std::io::prelude::*;
use std::ops;

#[derive(Copy, Clone)]
struct Vec3d {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3d {
    fn new(x: f64, y:f64, z:f64) -> Vec3d {
        Vec3d{x:x, y:y, z:z}
    }
    fn zero() -> Vec3d {
        Vec3d{x:0.0, y:0.0, z:0.0}
    }
}

impl ops::Add<Vec3d> for Vec3d {
    type Output = Vec3d;
    fn add(self, right: Vec3d) -> Vec3d {
        Vec3d::new(self.x + right.x, self.y + right.y, self.z + right.z)
    }
}

impl ops::Sub<Vec3d> for Vec3d {
    type Output = Vec3d;
    fn sub(self, right: Vec3d) -> Vec3d {
        Vec3d::new(self.x - right.x, self.y - right.y, self.z - right.z)
    }
}

impl ops::Div<Vec3d> for Vec3d {
    type Output = Vec3d;
    fn div(self, right: Vec3d) -> Vec3d {
        Vec3d::new(self.x / right.x, self.y / right.y, self.z / right.z)
    }
}

impl ops::Mul<Vec3d> for Vec3d {
    type Output = Vec3d;
    fn mul(self, right: Vec3d) -> Vec3d {
        Vec3d::new(self.x * right.x, self.y * right.y, self.z * right.z)
    }
}

impl ops::Mul<Vec3d> for f64 {
    type Output = Vec3d;
    fn mul(self, right: Vec3d) -> Vec3d {
        Vec3d::new(self * right.x, self * right.y, self * right.z)
    }
}

fn dot(a: Vec3d, b: Vec3d) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn squared_norm(v: Vec3d) -> f64 {
    dot(v, v)
}

fn norm(v: Vec3d) -> f64 {
    squared_norm(v).sqrt()
}

fn normalize(v: Vec3d) -> Vec3d {
    (1.0 / norm(v)) * v
}

#[derive(Copy, Clone)]
struct Sphere {
    position: Vec3d,
    squared_radius: f64,
    color: Vec3d,
}

#[derive(Copy, Clone)]
struct Light {
    direction: Vec3d,
    color: Vec3d,
}

struct World {
    spheres: Vec<Sphere>,
    lights: Vec<Light>,
    atmosphere_color: Vec3d,
}

#[derive(Copy, Clone)]
struct Intersection {
    position: Vec3d,
    normal: Vec3d,
    distance: f64,
    color: Vec3d,
}

fn make_intersection() -> Intersection {
    Intersection{
        position: Vec3d::zero(),
        normal: Vec3d::zero(),
        distance: f64::INFINITY,
        color: Vec3d::zero(),
    }
}

fn make_world() -> World {
    const R: f64 = 100000.0;
    const MAX_C: f64 = 1.0;
    const MIN_C: f64 = 0.1;
    let spheres = vec![
        Sphere{position:Vec3d::new(-2., 0., 6.), squared_radius:1., color: Vec3d::new(MAX_C, MAX_C, MIN_C)},
        Sphere{position:Vec3d::new(0., 0., 5.), squared_radius:1., color: Vec3d::new(MAX_C, MIN_C, MIN_C)},
        Sphere{position:Vec3d::new(2., 0., 4.), squared_radius:1., color: Vec3d::new(2.0*MIN_C, 4.0*MIN_C, MAX_C)},
        Sphere{position:Vec3d::new(0., 1.+R, 0.), squared_radius:R*R, color: Vec3d::new(MIN_C, MAX_C, MIN_C)},
        Sphere{position:Vec3d::new(0., -1.-R, 0.), squared_radius:R*R, color: Vec3d::new(MAX_C, MAX_C, MAX_C)},
    ];
    let lights = vec![
        Light{direction:Vec3d::new(1., 1., 2.), color:0.4 * Vec3d::new(1.0,0.8,0.5)},
        Light{direction:Vec3d::new(-1., -1., -2.), color:0.4 * Vec3d::new(0.5,0.5,1.0)},
    ];
    let atmosphere_color = 0.3 * Vec3d::new(0.5, 0.5, 1.0);
    return World{spheres, lights, atmosphere_color};
}

fn find_single_intersection(
    start: Vec3d, direction: Vec3d, sphere: Sphere
) -> Intersection {
    let mut intersection = make_intersection();
    let offset = sphere.position - start;
    let c = dot(direction, offset);
    if c < 0.0 {
        return intersection
    };
    let discriminant = c * c - squared_norm(offset) + sphere.squared_radius;
    if discriminant < 0.0 {
        return intersection
    } 
    intersection.distance = c - discriminant.sqrt();
    intersection.position = start + intersection.distance * direction;
    intersection.normal = normalize(intersection.position - sphere.position);
    intersection.color = sphere.color;
    return intersection;
}

fn find_intersection(
    start: Vec3d, direction: Vec3d, spheres: &Vec<Sphere>
) -> Intersection {
    let mut i1 = make_intersection();
    for sphere in spheres.iter() {
        let i2 = find_single_intersection(start, direction, *sphere);
        if i2.distance < i1.distance {
            i1 = i2
        }
    }
    return i1;
}

fn shade_single_light(intersection: Intersection, light: Light) -> Vec3d{
    let geometry = 0.0_f64.max(-dot(light.direction, intersection.normal));
    return geometry * (intersection.color * light.color)
}

fn shade_atmosphere(intersection: Intersection, atmosphere_color: Vec3d) -> Vec3d{
    intersection.position.z.sqrt() * atmosphere_color
}

fn shade(intersection: Intersection, world: &World) -> Vec3d {
    if intersection.distance.is_infinite() {
        return Vec3d::new(1., 1., 1.);
    }
    let mut color = shade_atmosphere(intersection, world.atmosphere_color);
    for light in world.lights.iter() {
        color = color + shade_single_light(intersection, *light);
    }
    return color;
}

fn color_u8_from_f64(c: f64) -> u8 {
    (255.0 * c).min(255.0) as u8
}

fn write_pixel(
    writer: &mut BufWriter<File>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    world: &World,
) {
    let start = Vec3d::zero();
    let xd = (x - width / 2) as f64;
    let yd = (y - height / 2) as f64;
    let zd = (height / 2) as f64;
    let direction = normalize(Vec3d::new(xd, yd, zd));
    let intersection = find_intersection(start, direction, &world.spheres);
    let color = shade(intersection, &world);
    let r = color_u8_from_f64(color.x);
    let g = color_u8_from_f64(color.y);
    let b = color_u8_from_f64(color.z);
    write!(writer, "{} {} {} ", r, g, b).unwrap();
}

fn write_image(file_path: &str, world: &World) {
    let file = File::create(file_path).unwrap();
    let mut writer = BufWriter::new(file);
    const WIDTH: i32 = 800;
    const HEIGHT: i32 = 600;
    write!(writer, "{}\n{}\n{}\n{}\n", "P3", WIDTH, HEIGHT, 255).unwrap();
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            write_pixel(&mut writer, x, y, WIDTH, HEIGHT, &world);
        }
    }
}

fn main() {
    println!("Saving image");
    let world = make_world();
    write_image("image.ppm", &world);
}
