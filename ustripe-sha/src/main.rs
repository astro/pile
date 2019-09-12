extern crate sha_1;

use std::env::args;
use std::thread;

mod ustripe;
use ustripe::*;

const GAMMA: f32 = 2.8;

trait Gamma {
    fn gamma(self) -> Self;
}

impl Gamma for f32 {
    fn gamma(self) -> Self {
        (255.0 * (self / 255.0).powf(GAMMA)).min(255.0)
    }
}

impl Gamma for u8 {
    fn gamma(self) -> Self {
        (self as f32).gamma() as u8
    }
}

impl<X: Clone + Gamma> Gamma for [X; 3] {
    fn gamma(self) -> Self {
        [self[0].clone().gamma(),
         self[1].clone().gamma(),
         self[2].clone().gamma(),
        ]
    }
}

fn mix_pixel(dest: &mut [f32; 3], src: &[f32; 3], alpha: f32) {
    for (i, d) in dest.iter_mut().enumerate() {
        *d = (1.0 - alpha) * *d + alpha * src[i] as f32;
    }
}

fn hash_to_colors(hash: &[u8; 20]) -> [[u8; 3]; LEDS] {
    let mut result = [[0; 3]; LEDS];

    for (x, mut rgb) in result.iter_mut().enumerate() {
        let i = 3 * ((hash.len() / 3) * x / LEDS);
        *rgb = [hash[i], hash[i + 1], hash[1 + 2]];
    }
    
    result
}

fn sha1(data: &[u8]) -> [u8; 20] {
    use sha_1::{Sha1, Digest};
    let mut hash = Sha1::default();
    hash.input(data);
    hash.hash()
}

fn main() {
    let lines = args().skip(1).collect::<Vec<String>>();
    
    let src = UstripeSource::new("10.0.0.1:2342", 26);

    let mut i = 0;
    let mut j = 1;
    let mut a = 0f32;
    loop {
        let from = hash_to_colors(&sha1(lines[i].as_bytes()));
        let mut to = hash_to_colors(&sha1(lines[j].as_bytes()));

        for (x, mut rgb) in to.iter_mut().enumerate() {
            let src = [from[x][0] as f32, from[x][1] as f32, from[x][2] as f32];
            let mut dst = [rgb[0] as f32, rgb[1] as f32, rgb[2] as f32];
            mix_pixel(&mut dst, &src, 1f32 - a);
            dst = dst.gamma();
            *rgb = [dst[0] as u8, dst[1] as u8, dst[2] as u8];
        }
        src.send(&to);

        a += 0.02;
        if a >= 1f32 {
            a = 0f32;
        }

        if a == 0f32 {
            i += 1;
            if i >= lines.len() {
                i = 0;
            }
            j = i + 1;
            if j >= lines.len() {
                j = 0;
            }
        }
        
        thread::sleep_ms(20);
    }
}
