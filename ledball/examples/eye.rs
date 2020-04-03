extern crate ledball;
extern crate haversine;
use std::thread::sleep;
use std::time::Duration;
use haversine::{Location, distance, Units};

use ledball::LedBall;

const PUPIL_DISTANCE: f64 = 2000.0;
const IRIS_DISTANCE: f64 = 4000.0;
const BORDER_DISTANCE: f64 = 4500.0;

pub fn main() {
    let l = LedBall::new("ledball1.hq.c3d2.de:2342", 0);
    let mut t = 0;
    loop {
        l.draw(|lat, lon| {
            let pos = Location {
                latitude: lat,
                longitude: lon,
            };
            let eye = Location {
                latitude: 0.0,
                longitude: (t % 360 - 180) as f64,
            };
            // println!("pos: {:.0}x{:.0}\teye: {:.0}x{:.0}", pos.longitude, pos.latitude, eye.longitude, eye.latitude);
            let d = distance(pos, eye, Units::Kilometers);
            // println!("d: {:.0}", d);
            if d <= PUPIL_DISTANCE {
                (0.0, 0.0, 0.0)
            } else if d <= IRIS_DISTANCE {
                (127.0, 63.0, 0.0)
            } else if d <= BORDER_DISTANCE {
                (0.0, 0.0, 0.0)
            } else {
                (255.0, 255.0, 255.0)
            }
        });
        t += 1;

        sleep(Duration::from_millis(20));
    }
}
