extern crate rand;

use rand::{thread_rng, Rng};
use std::thread::sleep;
use std::time::Duration;

mod batman;
use batman::*;

fn main() {
    let batman = BatmanSource::new("172.20.76.138:1234")
        .expect("BatmanSource");
    let mut rng = thread_rng();

    let mut color = [0, 0, 0u8];
    let mut half = Vec::with_capacity(LEDS / 2);
    for _ in 0..(LEDS / 2) {
        half.push([0, 0, 0u8]);
    }

    let mut pixels = Vec::with_capacity(LEDS);
    for t in 0usize.. {
        half.remove(0);
        let mut mid = color.clone();
        if t <= 3 {
            for c in mid.iter_mut() {
                *c = *c >> (3 - t);
            }
        } else {
            for c in mid.iter_mut() {
                *c = *c >> (t - 3);
            }
        }
        half.push(mid);
        let t1 = t % 13;
        if t1 == 0 {
            color = [rng.gen(), rng.gen(), rng.gen()];
        }
        
        pixels.clear();
        pixels.extend(&half[..]);
        pixels.push(color);
        pixels.extend(half.iter().rev());

        batman.send(&pixels[..]).expect("send");
        sleep(Duration::from_millis(25));
    }
}
