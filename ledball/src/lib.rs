use std::iter::Take;
use std::net::ToSocketAddrs;

mod ustripe;
pub use ustripe::{UstripeSource, LEDS};

pub struct LedBall {
    ustripe: UstripeSource,
}

pub type Color = (f64, f64, f64);

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
                    r.min(255.0).max(0.0) as u8,
                    g.min(255.0).max(0.0) as u8,
                    b.min(255.0).max(0.0) as u8,
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
    circle_index: usize,
    circle_progress: usize,
}

impl SphericalSpiralIterator {
    pub fn new() -> Self {
        SphericalSpiralIterator {
            circle_index: 0,
            circle_progress: 0,
        }
    }
}

const CIRCLES: &[usize] = &[48, 59, 70, 75, 80, 71, 61, 57, 42, 37, 26, 17, 31];

impl Iterator for SphericalSpiralIterator {
    /// (lat, lon)
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.circle_progress >= CIRCLES[self.circle_index] {
            self.circle_index += 1;
            self.circle_progress = 0;

            if self.circle_index >= CIRCLES.len() {
                return None;
            }
        }
        let rel_circle_progress = self.circle_progress as f64 / CIRCLES[self.circle_index] as f64;
        let lat = 180.0 * (1.0 + self.circle_index as f64 + rel_circle_progress) / (CIRCLES.len() + 1) as f64 - 90.0;
        let lon = 360.0 * rel_circle_progress - 180.0;

        self.circle_progress += 1;
        Some((lat, lon))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut circle_index = self.circle_index;
        let mut circle_progress = self.circle_progress;
        let mut size = 0usize;
        while circle_index < CIRCLES.len() {
            size += CIRCLES[circle_index] - circle_progress;
            circle_index += 1;
            circle_progress = 0;
        }
        (size, Some(size))
    }
}
