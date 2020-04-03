use std::iter::Take;
use std::net::ToSocketAddrs;

mod ustripe;
pub use ustripe::{UstripeSource, LEDS};

pub struct LedBall {
    ustripe: UstripeSource,
}

pub type Color = (f64, f64, f64);

const MAX: f64 = 127.0;

impl LedBall {
    pub fn new<A: ToSocketAddrs>(dest: A, priority: u8) -> Self {
        let ustripe = ustripe::UstripeSource::new(dest, priority);
        LedBall { ustripe }
    }

    pub fn leds(&self) -> usize {
        LEDS
    }

    pub fn pixel_coordinates() -> Take<SphericalSpiralIterator> {
        SphericalSpiralIterator::new().take(LEDS)
    }

    pub fn draw<F: FnMut(f64, f64) -> Color>(&self, mut f: F) {
        let pixels = Self::pixel_coordinates()
            .map(|(lat, lon)| {
                let (r, g, b) = f(lat, lon);
                let rgb: [u8; 3] = [
                    r.min(MAX).max(0.0) as u8,
                    g.min(MAX).max(0.0) as u8,
                    b.min(MAX).max(0.0) as u8,
                ];
                rgb
            })
            .collect::<Vec<_>>();
        self.send(&pixels);
    }

    pub fn send(&self, pixels: &[[u8; 3]]) {
        self.ustripe.send(pixels);
    }
}

pub struct SphericalSpiralIterator {
    n: usize,
}

impl SphericalSpiralIterator {
    pub fn new() -> Self {
        SphericalSpiralIterator {
            n: 0,
        }
    }
}

const CIRCLE_SIZE: f64 = 11.13;

impl Iterator for SphericalSpiralIterator {
    /// (lat, lon)
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.n >= LEDS {
            return None;
        }
        let circle_index = ((self.n as f64) % CIRCLE_SIZE) / CIRCLE_SIZE;
        let circle_progress = (self.n as f64) / (LEDS as f64);
        let lon = 360.0 * circle_index - 180.0;
        let lat = 180.0 * circle_progress - 90.0;

        self.n += 1;
        Some((lat, lon))
    }

    /*fn size_hint(&self) -> (usize, Option<usize>) {
        let mut circle_index = self.circle_index;
        let mut circle_progress = self.circle_progress;
        let mut size = 0usize;
        while circle_index < CIRCLES.len() {
            size += CIRCLES[circle_index] - circle_progress;
            circle_index += 1;
            circle_progress = 0;
        }
        (size, Some(size))
    }*/
}
