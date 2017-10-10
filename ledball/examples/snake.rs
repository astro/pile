extern crate ledball;
use std::thread::sleep;
use std::time::Duration;

use ledball::UstripeSource;


const INTERVAL: usize = 20;
const COLORS: &[[u8; 3]] = &[
    [63, 0, 0], [0, 63, 0], [0, 0, 63],
    // [0, 63, 63], [63, 0, 63], [63, 63, 0],
];

pub fn main() {
    let u = UstripeSource::new("ledball1:2342", 0);
    let mut t = 0;
    let mut pixels = [[0u8; 3]; ledball::LEDS];
    loop {
        for x in 0..pixels.len() {
            pixels[x] = COLORS[((x + t / 1) / INTERVAL) % COLORS.len()];
        }

        u.send(&pixels);
        println!("t={}", t);
        t += 1;

        sleep(Duration::from_millis(20));
    }
}
