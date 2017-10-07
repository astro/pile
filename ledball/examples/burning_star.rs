extern crate ledball;
extern crate rand;

use std::thread::sleep;
use std::time::Duration;
use rand::{thread_rng, Rng};

use ledball::UstripeSource;

#[derive(Debug, Clone)]
enum Pixel {
    Flare(u8),
    Burn(u8),
    Glow(u8),
}

impl Pixel {
    pub fn generate<R: Rng>(rng: &mut R) -> Self {
        let c: u16 = rng.gen_range(0, 768);
        if c > 511 {
            Pixel::Flare((c - 512) as u8)
        } else if c > 255 {
            Pixel::Burn((c - 256) as u8)
        } else {
            Pixel::Glow(c as u8)
        }
    }

    pub fn tick(&mut self) {
        match self {
            &mut Pixel::Flare(0) =>
                *self = Pixel::Burn(255),
            &mut Pixel::Flare(n) =>
                *self = Pixel::Flare(n - 1),

            &mut Pixel::Burn(0) =>
                *self = Pixel::Glow(255),
            &mut Pixel::Burn(n) =>
                *self = Pixel::Burn(n - 1),

            &mut Pixel::Glow(0) =>
                *self = Pixel::Flare(255),
            &mut Pixel::Glow(n) =>
                *self = Pixel::Glow(n - 1),
        }
    }

    pub fn current_color(&self) -> [u8; 3] {
        match self {
            &Pixel::Flare(n) =>
                [255 - n, 255 - n, 0],
            &Pixel::Burn(n) =>
                [255, n, 0],
            &Pixel::Glow(n) =>
                [n, 0, 0],
        }
    }
}


const TICKS_PER_ITERATION: usize = 4;

pub fn main() {
    let mut rng = thread_rng();
    let u = UstripeSource::new("ledball1:2342", 0);
    let mut pixels = Vec::with_capacity(ledball::LEDS);
    for _ in 0..ledball::LEDS {
        pixels.push(Pixel::generate(&mut rng));
    }
    loop {
        let rgb = pixels.iter()
            .map(|p| p.current_color())
            .collect::<Vec<_>>();
        u.send(&rgb);

        for x in 0..pixels.len() {
            for _ in 0..TICKS_PER_ITERATION {
                pixels[x].tick();
            }
        }
        sleep(Duration::from_millis(25));
    }
}
