extern crate haversine;
extern crate rand;
extern crate image;
extern crate ledball;

use std::thread::sleep;
use std::time::Duration;
use haversine::{Location, distance, Units};
use image::GenericImage;

use ledball::LedBall;


pub fn main() {
    let earth = image::open("earth.png").expect("image::open");
    
    let l = LedBall::new("ledball1:2342", 0);
    let mut t = 0;
    loop {
        l.draw(|lat, lon| {
            // let pos = Location {
            //     latitude: lat - 90.0,
            //     longitude: lon - 180.0,
            // };
            let ground = earth.get_pixel(
                (((lon - t as f64 / 2.0) % 360.0) * earth.width() as f64 / 360.0) as u32,
                (lat * earth.height() as f64 / 180.0) as u32
            );
            let m = |x: u8| ((x as f64) / 255.0).powf(1.2) * 255.0;

            (m(ground.data[0]),
             m(ground.data[1]),
             m(ground.data[2]),
            )
        });
        t += 1;

        sleep(Duration::from_millis(20));
    }
}
