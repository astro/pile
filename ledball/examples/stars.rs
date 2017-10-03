extern crate ledball;
extern crate rand;
use std::thread::sleep;
use std::time::Duration;
use rand::{thread_rng, Rng};

use ledball::UstripeSource;


pub fn main() {
    let u = UstripeSource::new("ledball1:2342", 0);
    let mut pixels = [[0u8; 3]; ledball::LEDS];
    let mut rng = thread_rng();
    loop {
        for x in 0..pixels.len() {
            let rgb = pixels[x];
            pixels[x] = [rgb[0].saturating_sub(1), rgb[1].saturating_sub(1), rgb[2].saturating_sub(1)];
        }
        let i = rng.gen_range(0, pixels.len());
        pixels[i] = [255, 255, 255];

        u.send(&pixels);

        sleep(Duration::from_millis(20));
    }
}
