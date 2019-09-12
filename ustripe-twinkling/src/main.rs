extern crate rand;

use std::cmp::min;
use std::thread;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

mod ustripe;
use ustripe::*;


fn main() {
    let src = UstripeSource::new("172.22.99.186:2342", 3);
    let mut rng = thread_rng();
    let mut data = [[0; 3]; LEDS];

    loop {
        // for d in data.iter_mut() {
        //     d[0] -= 4;
        //     d[1] -= 4;
        //     d[2] -= 4;
        // }
        
        let x = min(Range::new(0, LEDS).ind_sample(&mut rng), LEDS - 1);
        let w = min(Range::new(1, 5).ind_sample(&mut rng), LEDS - x);
        let rgb = [
            min(Range::new(0, 255).ind_sample(&mut rng), 255),
            min(Range::new(0, 255).ind_sample(&mut rng), 255),
            min(Range::new(0, 255).ind_sample(&mut rng), 255),
        ];
        for x1 in x..x+w {
            data[x1] = rgb;
        }

        src.send(&data);

        thread::sleep_ms(5);
    }
}
