extern crate ledball;
use std::thread::sleep;
use std::time::Duration;

use ledball::LedBall;


pub fn main() {
    let l = LedBall::new("ledball1:2342", 0);
    let mut t = 0;
    loop {
        l.draw(|lat, lon| {
            let a = (360.0 + lon - (t % 360) as f64) % 90.0;
            let b = (180.0 + lat - (t % 180) as f64) % 90.0;
            // println!("lat={:.0}\tlon={:.0}\ta={:.2}", lat, lon, a);
            if a < 15.0 && b < 20.0 {
                (255.0, 255.0, 255.0)
            } else if a < 15.0 {
                (255.0, 0.0, 0.0)
            } else if b < 20.0 {
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
