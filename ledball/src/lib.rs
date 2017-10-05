use std::net::ToSocketAddrs;

mod ustripe;
pub use ustripe::{UstripeSource, LEDS};

pub struct LedBall {
    ustripe: UstripeSource,
}

pub type Color = (f64, f64, f64);

const CIRCLES: &[usize] = &[48, 59, 70, 75, 80, 71, 61, 57, 42, 37, 26, 17, 31];

impl LedBall {
    pub fn new<A: ToSocketAddrs>(dest: A, priority: u8) -> Self {
        let ustripe = ustripe::UstripeSource::new(dest, priority);
        LedBall { ustripe }
    }

    pub fn draw<F: FnMut(f64, f64) -> Color>(&self, mut f: F) {
        let mut pixels = Vec::with_capacity(LEDS);
        let mut circle_index = 0;
        let mut circle_progress = 0;
        for _ in 0..LEDS {
            if circle_progress >= CIRCLES[circle_index] {
                circle_index += 1;
                circle_progress = 0;
            }

            let rel_circle_progress = circle_progress as f64 / CIRCLES[circle_index] as f64;
            let lon = 360.0 * rel_circle_progress;
            let lat = 180.0 * (1.0 + circle_index as f64 + rel_circle_progress) / (CIRCLES.len() + 1) as f64;

            let (r, g, b) = f(lat, lon);
            let rgb: [u8; 3] = [
                r.min(255.0).max(0.0) as u8,
                g.min(255.0).max(0.0) as u8,
                b.min(255.0).max(0.0) as u8,
            ];
            pixels.push(rgb);

            circle_progress += 1;
        }

        if circle_progress < CIRCLES[circle_index] {
            println!("{} LEDs left for this circle", CIRCLES[circle_index] - circle_progress);
        }
        self.ustripe.send(&pixels);
    }
}
