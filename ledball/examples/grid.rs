extern crate ledball;
use std::thread::sleep;
use std::time::Duration;

use ledball::LedBall;


pub fn main() {
    let l = LedBall::new("ledball1.hq.c3d2.de:2342", 0);
    let mut t = 0;
    loop {
        l.draw(|lat, lon| {
            let a = (lon + 180.0 + t as f64) % 360.0;
            let b = (lat + 90.0 + t as f64) % 180.0;
            // println!("lat={:.0}\tlon={:.0}\ta={:.2}", lat, lon, a);
            if a < 180.0 && b < 90.0 {
                (0.0, 255.0, 0.0)
            } else if a < 180.0 {
                (255.0, 0.0, 0.0)
            } else if b < 90.0 {
                (0.0, 0.0, 255.0)
            } else {
                (0.0, 0.0, 0.0)
            }
        });
        println!("t={}", t);
        t += 2;

        sleep(Duration::from_millis(20));
    }
}
