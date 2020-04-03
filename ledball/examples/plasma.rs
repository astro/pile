extern crate ledball;
use std::thread::sleep;
use std::time::Duration;
use std::f64::consts::PI;

use ledball::LedBall;

const TIMESCALE: f64 = 0.01;

pub fn main() {
    let l = LedBall::new("ledball1.hq.c3d2.de:2342", 0);
    let mut t = 0.0;
    loop {
        println!("t={}", t);
        l.draw(|lat, lon| {
            let lat = lat / 90.0;
            let lon = lon / 180.0;
            let t = t * TIMESCALE;
            let s1 = |a: f64| (lon * a + t).sin();
            let s2 = |a: f64, d1: f64, d2: f64| (a * (lon * (t / d1).sin() + lat * (t / d2).cos()) + t).sin();
            let s3 = |a: f64, d1: f64, d2: f64| {
                let cx = lon + 0.5 * (t / d1).sin();
                let cy = lat + 0.5 * (t / d2).sin();
                ((a * (cx.powi(2) + cy.powi(2)) + 1.0).sqrt() + t).sin()
            };
            let gamma = |c: f64| (255.0 * c).powf(1.2);
            let v = s1(0.2) + s2(2.0, 3.0, 5.0) + s3(2.0, 5.0, 8.0);
            (gamma((v * PI).sin()), gamma((v * PI).cos()), 0.0)
        });
        t += 0.01;

        sleep(Duration::from_millis(20));
    }
}
