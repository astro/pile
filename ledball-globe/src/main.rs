extern crate haversine;
extern crate rand;
extern crate image;
extern crate ledball;

use std::env::args;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;
use image::GenericImage;

use ledball::LedBall;


struct StaticPixelMap {
    /// For every LED have a list of pixel coordinates
    assocs: Vec<Vec<(u32, u32)>>,
}

impl StaticPixelMap {
    fn new<F: Fn(u32, u32) -> usize>(size: usize, w: u32, h: u32, f: F) -> Self {
        let mut assocs = (0..size).map(|_| Vec::new())
            .collect::<Vec<_>>();
        for y in 0..h {
            println!("y = {}", y);
            for x in 0..w {
                let i = f(x, y);
                assocs[i].push((x, y));
            }
        }

        StaticPixelMap { assocs }
    }

    fn get_texcoords_for_pixel(&self, i: usize) -> &[(u32, u32)] {
        &self.assocs[i]
    }
}

fn distance((lat1, lon1): (f64, f64), (lat2, lon2): (f64, f64)) -> f64 {
    use haversine::{Location, distance, Units};

    let pos1 = Location {
        latitude: lat1,
        longitude: lon1,
    };
    let pos2 = Location {
        latitude: lat2,
        longitude: lon2,
    };
    distance(pos1, pos2, Units::Kilometers)
}

pub fn main() {
    let args = args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} <texture-file>", args[0]);
        exit(1);
    }
    let texture = image::open(&args[1]).expect("image::open");
    let l = LedBall::new("ledball1:2342", 0);

    println!("Building map of {} pixels…", texture.width() * texture.height());
    let map = StaticPixelMap::new(l.leds(), texture.width(), texture.height(), |x, y| {
        let lon = 360.0 * (x as f64) / (texture.width() as f64) - 180.0;
        let lat = 180.0 * (y as f64) / (texture.height() as f64) - 90.0;
        let mut nearest = None;
        for (i, (p_lat, p_lon)) in LedBall::pixel_coordinates().enumerate() {
            match (nearest, distance((lat, lon), (p_lat, p_lon))) {
                (None, dist) =>
                    nearest = Some((i, dist)),
                (Some((_, nearest_dist)), dist) if dist < nearest_dist =>
                    nearest = Some((i, dist)),
                _ =>
                    (),
            }
        }
        nearest.expect("nearest").0
    });

    let mut t = 0;
    let mut pixels = (0..l.leds())
        .map(|_| [0, 0, 0])
        .collect::<Vec<_>>();
    loop {
        for (x, pixel) in pixels.iter_mut().enumerate() {
            let mut size = 0usize;
            let mut r = 0usize;
            let mut g = 0usize;
            let mut b = 0usize;
            for &(x, y) in map.get_texcoords_for_pixel(x) {
                let ground = texture.get_pixel((x - t) % texture.width(), y);
                r += ground.data[0] as usize;
                g += ground.data[1] as usize;
                b += ground.data[2] as usize;
                size += 1;
            }
            if size > 0 {
                *pixel = [
                    (r / size) as u8,
                    (g / size) as u8,
                    (b / size) as u8,
                ];
            } else {
                println!("LED {} stays black :(", x);
            }
        }
        l.send(&pixels);
        t += 1;

        sleep(Duration::from_millis(20));
    }
}
