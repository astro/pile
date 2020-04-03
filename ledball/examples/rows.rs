extern crate ledball;
extern crate rand;

use std::thread::sleep;
use std::time::Duration;
use rand::{thread_rng, Rng};

use ledball::UstripeSource;


const ROW_LENGTH: usize = 40;
const INTERVAL: u64 = 25;
const STEPS_PER_TICK: usize = 1;

pub fn main() {
    let mut rng = thread_rng();
    let u = UstripeSource::new("ledball1.hq.c3d2.de:2342", 1);
    let mut current = [0, 0, 0];
    let mut pixels = vec![current; ledball::LEDS];
    let mut t = 0;
    loop {
        u.send(&pixels);

        for _ in 0..STEPS_PER_TICK {
            t += 1;
            if t % ROW_LENGTH == 0 {
                match rng.gen_range(0, 6) {
                    0 =>
                        current = [
                            rng.gen_range(0, 127),
                            0,
                            0,
                        ],
                    1 =>
                        current = [
                            0,
                            rng.gen_range(0, 127),
                            0,
                        ],
                    2 =>
                        current = [
                            0, 0,
                            rng.gen_range(0, 127),
                        ],
                    3 =>
                        current = [
                            0,
                            rng.gen_range(0, 63),
                            rng.gen_range(0, 63),
                        ],
                    4 =>
                        current = [
                            rng.gen_range(0, 63),
                            0,
                            rng.gen_range(0, 63),
                        ],
                    5 =>
                        current = [
                            rng.gen_range(0, 63),
                            rng.gen_range(0, 63),
                            0,
                        ],
                    _ => unreachable!(),
                }
            }

            pixels.remove(0);
            pixels.push(current.clone());
        }

        sleep(Duration::from_millis(INTERVAL));
    }
}
